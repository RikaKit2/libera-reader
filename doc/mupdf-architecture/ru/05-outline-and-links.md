# 05. Оглавление и гиперссылки (Outline & Links)

Данный документ описывает механизмы извлечения структуры оглавления (TOC / Bookmarks), обработки внутренних переходов по документу и внешних гиперссылок.

---

## 1. Дерево оглавления (Document Outline / Table of Contents)

### 1.1. Извлечение через MuPDF
В MuPDF оглавление документа загружается методом:
```javascript
let outline = doc.loadOutline();
```
Каждый узел оглавления содержит:
- `title`: название главы/секции.
- `page`: номер целевой страницы (0-indexed).
- `uri`: опциональный URI (если ссылка ведёт наружу).
- `down`: дочерние элементы (подсекции).
- `next`: следующий элемент того же уровня вложенности.

В Rust эта модель определена в `crates/mutool/src/outline.rs`:
```rust
pub struct OutlineItem {
    pub title: String,
    pub page: Option<usize>,
    pub uri: Option<String>,
    pub children: Vec<OutlineItem>,
}
```

### 1.2. Отображение в боковой панели GPUI Kit
- Оглавление отображается во вкладке сайдбара `SidebarTab::Outline` (`crates/libera-reader/src/ui/pages/book-viewer/sidebar/tabs/outline.rs`).
- Для иерархических структур в GPUI Kit используется компонент `Tree` (`gpui_kit::component::tree::{Tree, TreeState, TreeItem}`) либо стилизованный рекурсивный список.
- Клик по элементу:
  ```rust
  if let Some(target_page) = item.page {
      state.update(cx, |s, cx| {
          s.scroll_to_page(target_page);
          cx.notify();
      });
  }
  ```
- Текущая читаемая страница может подсвечивать активный пункт оглавления в дереве.

---

## 2. Ссылки на странице (Page Links)

### 2.1. Извлечение через MuPDF
Ссылки страницы извлекаются через `page.getLinks()`:
```javascript
let links = page.getLinks();
links.map((link) => {
    const [ x0, y0, x1, y1 ] = link.getBounds();
    let href = link.isExternal() ? link.getURI() : `#page${doc.resolveLink(link) + 1}`;
    return { x: x0, y: y0, w: x1 - x0, h: y1 - y0, href };
});
```

В Rust модель представлена в `crates/mutool/src/links.rs`:
```rust
pub struct PageLink {
    pub bbox: BBox,              // Координаты в PDF-пунктах (72 DPI)
    pub dest_page: Option<usize>, // 1-indexed номер страницы для внутреннего перехода
    pub uri: Option<String>,      // URL для внешнего перехода
}
```

### 2.2. Отрисовка в `PageLinksLayer`
- Каждый `PageLink` позиционируется поверх `PageCanvas` через `bbox.scaled(zoom_factor)`.
- Слой `PageLinksLayer` (`crates/libera-reader/src/ui/pages/book-viewer/viewport/page/page_links_layer.rs`):
  - При наведении курсор меняется на `CursorStyle::PointingHand`.
  - При `hover` выводится мягкая рамка с цветом `cx.theme().primary.opacity(0.35)`.
  - Клик по внутренней ссылке:
    ```rust
    if let Some(dest) = dest_page {
        state.update(cx, |s, cx| {
            s.scroll_to_page(dest);
            cx.notify();
        });
    }
    ```
  - Клик по внешней ссылке: вызов системного открытия URL через `cx.open_url(&uri)`.

### 2.3. Изоляция от выделения текста мышью
- Чтобы клики по ссылкам не блокировали выделение фрагментов текста, `PageLinksLayer` реагирует на чистое нажатие без движения (`mouse_up` в пределах начальной точки), в то время как перемещение мыши с зажатой кнопкой передаётся в диспетчер `TextSelection`.
