# План реализации: Персональный кэш книги, ограничение нагрузки CPU и самовосстановление (Book Cache & Concurrency Plan)

> **Для исполнителей:** ОБЯЗАТЕЛЬНЫЙ НАВЫК: Используйте `superpowers:subagent-driven-development` (рекомендуется) или `superpowers:executing-plans` для пошаговой реализации плана. Шаги используют синтаксис чекбоксов (`- [ ]`).

**Цель:** Устранить 100% загрузку CPU при открытии книги за счёт ограничения параллелизма `mutool`, внедрить изолированный файловый кэш страниц и обложек для каждой книги на диске, обеспечить автоматическое самовосстановление кэша при его отсутствии или повреждении и чистую очистку всей папки при удалении книги.

**Архитектура:** Для каждой книги создаётся изолированная папка `~/.local/share/libera-reader/thumbnails/{hashed_books/<hash> | unhashed_books/<size>}/`, в которой хранятся `cover.png` и подпапка `pages/p{page}_{dpi}.png`. Семафор `mutool` на 2-ядерных CPU ограничивается 2 потоками. Перед рендерингом всегда проверяется наличие и целостность файла на диске: если файл есть, он загружается мгновенно без вызова `mutool`; если файл отсутствует (даже если база считает, что он есть) — запускается самовосстановление через `mutool`. При удалении книги папка удаляется целиком в один вызов `std::fs::remove_dir_all`.

**Стек:** Rust 1.98.1, GPUI Kit 0.7.0, Tokio, MuPDF CLI (`mutool`), `native_db`.

## Глобальные ограничения

- **Запрет на тяжелые сборки:** Запрещено запускать `cargo test`, `cargo build` или `cargo clippy` синхронно без ограничения потоков. Использовать `cargo fix-all` для проверки.
- **Git коммиты:** Только по прямому запросу пользователя, с флагом `-S` (GPG подпись) и на английском языке в формате Conventional Commits.
- **Никаких бинарных картинок в базе данных:** Изображения хранятся строго файлами на диске. В базе хранятся только метаданные, пути и хэши.

---

### Задача 1: Расширение `AppDirs` — единая структура директорий для книги

**Файлы:**
- Изменить: `crates/libera-reader/src/app_dirs.rs`
- Тест: `crates/libera-reader/src/app_dirs.rs` (модульные тесты)

**Интерфейсы:**
- Потребляет: `book_size: u64`, `hash_opt: Option<&str>`
- Производит:
  - `AppDirs::book_dir(&self, book_size: u64, hash_opt: Option<&str>) -> PathBuf`
  - `AppDirs::book_cover_path(&self, book_size: u64, hash_opt: Option<&str>) -> PathBuf`
  - `AppDirs::book_pages_dir(&self, book_size: u64, hash_opt: Option<&str>) -> PathBuf`
  - `AppDirs::book_page_path(&self, book_size: u64, hash_opt: Option<&str>, page: usize, dpi: u32) -> PathBuf`

- [ ] **Шаг 1: Написать тест на пути кэша книги**

В `crates/libera-reader/src/app_dirs.rs`:
```rust
#[cfg(test)]
mod tests {
  use super::*;
  use tempfile::tempdir;

  #[test]
  fn test_book_paths_generation() {
    let tmp = tempdir().unwrap();
    let dirs = Dirs::new(tmp.path().to_path_buf()).unwrap();
    let app_dirs = AppDirs { inn: Arc::new(dirs) };

    // Хэшированная книга
    let hash_dir = app_dirs.book_dir(1024, Some("a1b2c3d4"));
    assert!(hash_dir.ends_with("thumbnails/hashed_books/a1b2c3d4"));
    let cover_path = app_dirs.book_cover_path(1024, Some("a1b2c3d4"));
    assert!(cover_path.ends_with("thumbnails/hashed_books/a1b2c3d4/cover.png"));
    let page_path = app_dirs.book_page_path(1024, Some("a1b2c3d4"), 5, 150);
    assert!(page_path.ends_with("thumbnails/hashed_books/a1b2c3d4/pages/p5_150dpi.png"));

    // Нехэшированная книга
    let unhash_dir = app_dirs.book_dir(2048, None);
    assert!(unhash_dir.ends_with("thumbnails/unhashed_books/2048"));
  }
}
```

- [ ] **Шаг 2: Реализовать методы в `AppDirs` и `Dirs`**

```rust
impl Dirs {
  pub fn book_dir(&self, book_size: u64, hash_opt: Option<&str>) -> PathBuf {
    match hash_opt {
      Some(hash) if !hash.is_empty() => self.dir_of_hashed_books.join(hash),
      _ => self.dir_of_unhashed_books.join(book_size.to_string()),
    }
  }

  pub fn book_cover_path(&self, book_size: u64, hash_opt: Option<&str>) -> PathBuf {
    self.book_dir(book_size, hash_opt).join("cover.png")
  }

  pub fn book_pages_dir(&self, book_size: u64, hash_opt: Option<&str>) -> PathBuf {
    self.book_dir(book_size, hash_opt).join("pages")
  }

  pub fn book_page_path(
    &self, book_size: u64, hash_opt: Option<&str>, page: usize, dpi: u32,
  ) -> PathBuf {
    self.book_pages_dir(book_size, hash_opt).join(format!("p{}_{}dpi.png", page, dpi))
  }
}
```

- [ ] **Шаг 3: Проверить через `cargo fix-all`**

---

### Задача 2: Ограничение параллелизма `mutool` и предотвращение 100% CPU

**Файлы:**
- Изменить: `crates/libera-reader/src/ui/pages/book-viewer/loader.rs:30-40`
- Изменить: `crates/libera-reader/src/services/data_extraction_service.rs:20-25`

**Интерфейсы:**
- Потребляет: физические ядра процессора (`num_cpus::get_physical()`)
- Производит: Ограничение пула воркеров максимум до 2 потоков на 2-ядерных CPU

- [ ] **Шаг 1: Ограничить количество одновременных `mutool` в `loader.rs`**

В `crates/libera-reader/src/ui/pages/book-viewer/loader.rs`:
```rust
  // На 2-ядерном CPU (4 потока) нельзя запускать 4 тяжелых процесса mutool одновременно.
  // Ограничиваем пул до числа физических ядер (максимум 2-3).
  let physical_cores = num_cpus::get_physical();
  let thread_count = physical_cores.clamp(1, 2);
  let semaphore = Arc::new(tokio::sync::Semaphore::new(thread_count));
```

- [ ] **Шаг 2: Устранить тройной запуск `mutool` на каждый кадр**
  - Объединить получение структурированного текста и ссылок или кэшировать их один раз в `BookViewerCache` без перезапуска `mutool` при зуме.
  - Текстовый слой `stext` и ссылки `links` извлекаются **один раз на страницу на весь сеанс книги** (они не зависят от DPI растра).

- [ ] **Шаг 3: Проверить через `cargo fix-all`**

---

### Задача 3: Персистентный дисковый кэш страниц в `BookViewer` и самовосстановление

**Файлы:**
- Изменить: `crates/libera-reader/src/ui/pages/book-viewer/loader.rs:65-105`
- Изменить: `crates/libera-reader/src/ui/pages/book-viewer/state/mod.rs` (передача размера и хэша книги в `PageLoadRequest`)

**Интерфейсы:**
- Потребляет: `PageLoadRequest { page, book_path, dpi, book_size, book_hash }`
- Производит: Проверку наличия файла `book_page_path` на диске перед вызовом `mutool`, валидацию целостности (не пустой файл) и авто-регенерацию при ошибке.

- [ ] **Шаг 1: Добавить проверку наличия файла на диске перед запуском `mutool`**

В `tokio::task::spawn_blocking` в `loader.rs`:
```rust
  let page_file = app_dirs.book_page_path(req.book_size, req.book_hash.as_deref(), page, dpi);

  // 1. Проверяем наличие валидного файла кэша на диске
  let disk_image = if page_file.exists() {
    match std::fs::read(&page_file) {
      Ok(bytes) if !bytes.is_empty() => decode_page_image_bytes(&bytes),
      _ => {
        // Файл повреждён или имеет нулевой размер — удаляем битый файл
        let _ = std::fs::remove_file(&page_file);
        None
      }
    }
  } else {
    None
  };

  // 2. Если файл есть на диске — используем его без запуска mutool (0% CPU!)
  let img_state = if let Some(img) = disk_image {
    PageImageState::Loaded(img)
  } else {
    // 3. Самовосстановление: файла нет или он был битый — запускаем mutool
    match mutool::render_page_to_png_bytes(&path, page, dpi) {
      Ok(bytes) => {
        // Сохраняем отрендеренную страницу на диск для будущих открытий
        if let Some(parent) = page_file.parent() {
          let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&page_file, &bytes);

        match decode_page_image_bytes(&bytes) {
          Some(img) => PageImageState::Loaded(img),
          None => PageImageState::Failed("Failed to decode PNG bytes".to_string()),
        }
      }
      Err(err) => PageImageState::Failed(err.to_string()),
    }
  };
```

- [ ] **Шаг 2: Проверить через `cargo fix-all`**

---

### Задача 4: Проверка обложки книги и самовосстановление (`Self-Healing Cover Check`)

**Файлы:**
- Изменить: `crates/libera-reader/src/db/models/books/book/mod.rs`
- Изменить: `crates/libera-reader/src/services/data_extraction_service.rs`
- Изменить: `crates/libera-reader/src/books_state/thumbnails.rs`

**Интерфейсы:**
- Потребляет: Метаданные книги в БД
- Производит: Проверку реального существования файла `cover.png` на диске. Если в БД стоит отметка, что миниатюра извлечена, но физический файл отсутствует или повреждён — автоматически запланировать извлечение через `mutool`.

- [ ] **Шаг 1: Написать метод валидации обложки на диске**

В `crates/libera-reader/src/db/models/books/book/mod.rs`:
```rust
  /// Проверяет физическое существование валидного (непустого) PNG файла на диске.
  pub fn is_thumbnail_valid_on_disk(&self, db: &crate::db::DB, app_dirs: &crate::app_dirs::AppDirs) -> bool {
    let hash = self.get_hash(db);
    let BookSize::BYTES(size) = self.book_size;
    let cover_path = app_dirs.book_cover_path(size, hash.as_deref());
    
    // Проверяем как новый путь (cover.png в папке книги), так и легаси fallback (размер.png)
    if cover_path.exists() && std::fs::metadata(&cover_path).map(|m| m.len() > 0).unwrap_or(false) {
      return true;
    }
    let fallback = app_dirs.dir_of_unhashed_books.join(format!("{}.png", size));
    fallback.exists() && std::fs::metadata(&fallback).map(|m| m.len() > 0).unwrap_or(false)
  }
```

- [ ] **Шаг 2: Самовосстановление в `thumbnails.rs`**
  Если `book.is_thumbnail_valid_on_disk(...)` возвращает `false`, сервис фоновой загрузки повторно отправляет задачу в `data_extraction_service`, восстанавливая отсутствующий PNG через `mutool`.

- [ ] **Шаг 3: Проверить через `cargo fix-all`**

---

### Задача 5: Полная атомарная очистка кэша книги при её удалении

**Файлы:**
- Изменить: `crates/libera-reader/src/services/notify_service/fs_handlers/remove_book.rs`
- Изменить: `crates/libera-reader/src/services/notify_service/fs_handlers/remove_books_in_dir.rs`

**Интерфейсы:**
- Потребляет: `book: &Book`, `app_dirs: &AppDirs`
- Производит: Удаление всей директории `book_dir` книги (`cover.png` + `pages/*`) в один системный вызов.

- [ ] **Шаг 1: Добавить удаление директории кэша книги при удалении записи**

В `crates/libera-reader/src/services/notify_service/fs_handlers/remove_book.rs`:
```rust
  if can_delete {
    // 1. Физически удаляем всю папку кэша этой книги со всеми страницами и обложкой
    let BookSize::BYTES(size) = book.book_size;
    let hash = book.get_hash_from_db(rw_t);
    let book_cache_dir = app_dirs.book_dir(size, hash.as_deref());
    if book_cache_dir.exists() {
      let _ = std::fs::remove_dir_all(&book_cache_dir);
    }

    // 2. Удаляем запись из BookSizes и Book
    BookSizes::remove_book(book.book_size, &book.book_path, rw_t)?;
    rw_t.remove::<Book>(book)?;
    Ok(RemoveStatus::FullyDeleted)
  }
```

- [ ] **Шаг 2: Проверить через `cargo fix-all`**

---

### Верификация

1. **Компиляция**: Запуск `cargo fix-all` — 0 ошибок, 0 предупреждений.
2. **Проверка CPU**: Открытие книги — процессор не бьётся в 100%, интерфейс отзывчив (воркеры ограничены 2 потоками).
3. **Проверка кэша**: Первое открытие рендерит страницу в качественном DPI $\to$ повторное открытие или скролл назад загружает готовую страницу за 1 мс без запуска `mutool`.
4. **Проверка самовосстановления**: При ручном удалении файла из `thumbnails/` приложение автоматически определяет нехватку файла и пересоздаёт его через `mutool`.
5. **Проверка удаления**: При удалении книги папка `thumbnails/{hashed|unhashed}/...` исчезает целиком.
