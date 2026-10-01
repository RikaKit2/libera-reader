# План реализации: Сжатие обложек, асинхронная подгрузка картинок минуя БД и надежное скачивание mutool

> **Для исполнителей:** ОБЯЗАТЕЛЬНЫЙ НАВЫК: Используйте `superpowers:subagent-driven-development` (рекомендуется) или `superpowers:executing-plans` для пошаговой реализации плана. Шаги используют синтаксис чекбоксов (`- [ ]`).

**Цель:**
1. Сократить дисковый объем обложек книг в 5–10 раз (с 200–600 КБ в PNG до 25–40 КБ в JPEG с качеством 85% / WebP) и ускорить их декодирование в интерфейсе GPUI.
2. Реализовать асинхронную подгрузку обложек в интерфейс напрямую с диска, исключив паразитные блокировки транзакций базы данных (`redb` / `native_db`).
3. Решить проблему скачивания и запуска `mutool`: заменить временные протухающие ссылки Uptodown на официальные релизы Artifex Software на GitHub (`mupdf-1.28.5-windows.zip`), исправить баг пропуска скачивания при отсутствии директории и централизовать поиск/запуск исполняемого файла `mutool` через единый резолвер команды.

**Стек:** Rust 1.98.1, GPUI Kit 0.7.0, Tokio, image 0.25 (JPEG quality 85), MuPDF CLI (`mutool`), `native_db`.

---

## Глобальные ограничения

- **Запрет на тяжелые сборки:** Запрещено запускать `cargo test`, `cargo build` или `cargo clippy` синхронно без ограничения потоков. Использовать `hub start` с флагом `-j 2`.
- **Git коммиты:** Только по прямому запросу пользователя, с флагом `-S` (GPG подпись) и на английском языке в формате Conventional Commits.
- **Никаких картинок в БД:** В базе данных хранятся строго метаданные и пути. Сами изображения хранятся на диске.

---

### Задача 1: Сжатие обложек (JPEG с качеством 85% / WebP) и обратная совместимость путей

**Файлы:**
- Изменить: `crates/mutool/src/extract_img.rs`
- Изменить: `crates/libera-reader/src/app_dirs.rs`
- Изменить: `crates/libera-reader/src/db/models/books/book/mod.rs`
- Тест: `crates/libera-reader/src/app_dirs.rs`

**Шаги:**
- [x] **Шаг 1.1:** Обновить `extract_img` в `crates/mutool/src/extract_img.rs`:
  - Рендерить страницу 1 через `mutool draw` во временный PNG файл.
  - Декодировать через `image::ImageReader` и кодировать в JPEG с качеством 85% (`image::codecs::jpeg::JpegEncoder::new_with_quality(..., 85)`).
  - Сохранять результат непосредственно в целевой файл `cover.jpg`.
- [x] **Шаг 1.2:** Обновить `app_dirs.rs`:
  - `book_cover_path` возвращает путь к `cover.jpg`.
  - Добавить методы поддержки форматов с проверкой существования: приоритет `cover.jpg` -> `cover.webp` -> `cover.png` -> legacy `<id>.png`.
- [x] **Шаг 1.3:** Обновить резолвер обложки в `Book::get_thumbnail_path`:
  - Бесшовная обратная совместимость: если у пользователя уже есть `cover.png` или legacy `<id>.png`, они продолжают отображаться без необходимости повторного извлечения.

---

### Задача 2: Асинхронная подгрузка картинок в UI минуя блокировки БД

**Файлы:**
- Изменить: `crates/libera-reader/src/books_state/thumbnails.rs`
- Изменить: `crates/libera-reader/src/books_state/mod.rs`
- Изменить: `crates/libera-reader/src/ui/components/books_grid/mod.rs`
- Изменить: `crates/libera-reader/src/ui/components/books_grid/loader.rs`

**Шаги:**
- [x] **Шаг 2.1:** Исключить вызовы транзакций `db` при сборе путей к обложкам в UI:
  - Формировать ожидаемый путь к обложке на диске (`book_dir/cover.jpg`) напрямую из `BookSize` и закэшированного хэша книги в оперативной памяти.
- [x] **Шаг 2.2:** Убедиться, что фоновый загрузчик (`spawn_background_loader`) читает файлы с диска изолированно в пуле потоков без захвата блокировок базы данных.

---

### Задача 3: Надежное скачивание и кроссплатформенный запуск `mutool`

**Файлы:**
- Изменить: `crates/mutool/src/download_mutool.rs`
- Изменить: `crates/mutool/src/lib.rs`
- Изменить: `crates/mutool/src/extract_img.rs`
- Изменить: `crates/mutool/src/render_page.rs`
- Изменить: `crates/mutool/src/page_info.rs`
- Изменить: `crates/mutool/src/stext.rs`
- Изменить: `crates/mutool/src/links.rs`
- Изменить: `crates/mutool/src/outline.rs`
- Изменить: `crates/mutool/src/search.rs`
- Изменить: `crates/libera-reader/src/main.rs`

**Шаги:**
- [x] **Шаг 3.1:** Исправить URL и логику в `download_mutool.rs`:
  - Для Windows: использовать официальный постоянный релиз Artifex Software на GitHub:
    `https://github.com/ArtifexSoftware/mupdf-downloads/releases/download/1.28.5/mupdf-1.28.5-windows.zip`.
  - Устранить баг `if path_to_mutool_storage.exists()`: всегда создавать директорию через `create_dir_all` перед проверкой наличия бинарника.
- [x] **Шаг 3.2:** Внедрить единый резолвер исполняемого файла `mutool::mutool_command()`:
  - Вместо разрозненных `Command::new("mutool")` вызывать централизованную функцию.
  - Резолвер проверяет:
    1) Переменную окружения `MUTOOL_PATH` (если задана разработчиком/пользователем).
    2) Наличие `mutool` / `mutool.exe` в системном `PATH`.
    3) Наличие локально скачанного `mutool` в папке данных приложения (`AppDirs::mutool`).
- [x] **Шаг 3.3:** Подключить проверку `download_mutool_if_missing_blocking` в `main.rs` до запуска сервисов и рендера.

---

### Задача 4: Верификация и обновление документации

- [x] **Шаг 4.1:** Запустить компиляцию и тесты через `cargo test` с ограничением `-j 2`.
- [x] **Шаг 4.2:** Проверить создание JPEG обложки и замерить размер файла (ожидается 25–40 КБ вместо 200–600 КБ).
- [x] **Шаг 4.3:** Обновить статусы в `docs/STATUS.md` и `docs/COMPLETED.md`.
