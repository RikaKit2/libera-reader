# Выполненные задачи и история разработки (Completed Work)

В данном документе фиксируется история завершённых архитектурных и прикладных задач в проекте **Libera Reader**.

---

## 1. Архитектура и документация MuPDF & GPUI Kit
- [x] **Исследование эталонных реализаций**:
  - Полный анализ `other/mupdf.js` (unminified `simple-viewer`, `worker.js`, `style.css`).
  - Исследование `other/mupdf-webviewer` (SDK `v0.15.0`, `types.d.ts` с полной типизацией API).
  - Сравнение с клиентскими обёртками `mupdf-webviewer-vue-sample` и `mupdf-webviewer-vanilla-js-sample`.
- [x] **Создание двуязычной архитектурной спецификации (`docs/mupdf-architecture/`)**:
  - `01-coordinate-spaces-and-dpi.md`: 4 координатных пространства (PDF-пункты, GPUI `px`, Physical Display, Pixmap Buffer), формулы взаимного перевода, двухуровневый кэш растра.
  - `02-page-layer-stack.md`: иерархия слоёв страницы (`PageView`), Z-index, бесконфликтная маршрутизация событий мыши (ссылки vs выделение).
  - `03-text-layer-and-selection.md`: структура `PageStructuredText`, физика шрифтового дрейфа (Font Metric Mismatch), решение MuPDF (SVG `lengthAdjust`) и перенос в `gpui_base::TextSelection`.
  - `04-search-engine.md`: C-движок MuPDF `toStructuredText().search()`, Quad-полигоны, автомат состояний `BookViewerState`, циклическая навигация `Enter`/`Shift+Enter`.
  - `05-outline-and-links.md`: дерево оглавления (`loadOutline()`), внутренние переходы (`resolveLink`) и внешние URI.
  - `06-annotations.md`: спецификация ISO 32000 (Highlight, Underline, StrikeOut, FreeText, Stamp, Note), двухэтапный план внедрения.
  - Полная синхронизация версий в директориях `docs/mupdf-architecture/en/` и `docs/mupdf-architecture/ru/` с двуязычным корневым `README.md`.
- [x] **Унификация документации**:
  - Устранено дублирование папок `doc/` и `docs/`, всё централизовано в каноничной папке `docs/`.

---

## 2. Подсистема рендеринга и расчет DPI (HiDPI & Zoom)
- [x] **Устранение жестко захардкоженного `PAGE_RENDER_DPI = 150`**:
  - Реализована чистая функция `compute_target_dpi(zoom_factor, scale_factor)` с границами $36{-}300\text{ DPI}$.
  - Поддержка HiDPI экранов (Retina / 4K) через автоматический учёт `window.scale_factor()`.
- [x] **Двухуровневое кэширование растра (Phase 1 + Phase 2)**:
  - В `BookViewerCache` внедрён учет разрешений `image_dpis: LruCache<usize, u32>` с методами `get_image_dpi`, `set_image_dpi`, `clear_images`.
  - При смене зума уже имеющаяся в памяти картинка мгновенно растягивается на GPU через GPUI `img()` (нет белых экранов и лагов).
  - В фоне асинхронно запрашивается рендер в целевом физическом разрешении экрана, который бесшовно заменяет старую текстуру на кристально чёткую.
  - Интеграция в оба режима просмотра: `scroll_container.rs` (непрерывный скролл) и `paged_container.rs` (постраничный режим).

---

## 3. Текстовый слой и полнотекстовый поиск (Search & Selection)
- [x] **Устранение шрифтового дрейфа (Font Metric Mismatch)**:
  - Полностью вырезан старый эвристический fallback живого поиска, рассчитывавший границы через системный шрифт `Inter`.
  - Устранены баги с обрезанием слов («Вы смогл» вместо «Вы смогли») и наплывом подсветки на соседние буквы («Вы смогли с|»).
  - Отрисовка подсветки переведена строго на C-движок MuPDF `search_document_text` с точными координатами `BBox`.
  - Активное совпадение подсвечивается ярким токеном `cx.theme().primary.opacity(0.65)` с контрастной рамкой `primary`, пассивные — `primary.opacity(0.20)` с рамкой `primary.opacity(0.40)`.
- [x] **Автомат состояний поиска в `BookViewerState`**:
  - Введён признак `has_searched`: исключён ложный статус «Не найдено» во время набора текста до подтверждения поиска.
  - Добавлена индикация асинхронного сканирования (`is_searching`).
  - Реализована циклическая навигация: `Enter` / стрелка `>` для перехода к следующему совпадению, `Shift+Enter` / стрелка `<` для перехода к предыдущему.
  - Синхронизация вьюпорта: при смене активного совпадения виртуальный список автоматически скроллится к целевой странице (`scroll_to_item`).

---

## 4. Очистка кодовой базы и миграция зависимостей
- [x] **Удаление устаревшего мёртвого кода**:
  - Файл `crates/libera-reader/src/ui/pages/book-viewer/viewport/page/page_search_layer.rs` удален из репозитория.
  - Модули `page/mod.rs`, `paged_container.rs` и `scroll_container.rs` очищены от устаревших импортов.
- [x] **Миграция на `gpui-kit` (v0.6.4)**:
  - Устранены устаревшие git-зависимости (`zed-industries/zed` и `longbridge/gpui-component`).
  - Подключён официальный релиз с Crates.io: `gpui-kit = "0.6.4"`, `gpui-pre = "0.3.5"`, `gpui-base = "0.6.4"`, `gpui-component = "0.6.4"`, `gpui-kit-assets = "0.6.4"`.
  - В корневом `Cargo.toml` прописаны настройки профиля `[profile.dev.package]` для максимальной производительности в dev-режиме (`opt-level = 3` для внутренних крейтов GPUI и парсеров шрифтов).
  - Настроен фасад совместимости в `crates/libera-reader/src/lib.rs` (`pub use gpui_kit;`).

---

## 5. Оптимизация окружения и профилей сборки
- [x] **Аудит глобального `cargo_config.toml`**:
  - Изучен глобальный конфиг `/home/user/nixos-config/modules/software/utils/cargo_config.toml`.
  - Выявлено и устранено бутылочное горлышко `debug = 2`: переход на `debug = "line-tables-only"` и `split-debuginfo = "unpacked"`.
  - Очищен кэш `/mnt/cache/build_dir` (освобождено 14 ГБ дискового пространства).
  - Проект стабилизирован на компиляторе **Rust Stable 1.98.1** с высокой кадровой частотой (FPS) даже в dev-сборках.
