# 01. Системы координат, масштаб (Zoom) и HiDPI (DPI)

Данный документ формализует системы координат, используемые при отображении PDF/EPUB документов в MuPDF, веб-эталоне (`other/mupdf.js`) и десктопном приложении на базе GPUI Kit (`crates/libera-reader` и `crates/mutool`).

---

## 1. Фундаментальные системы координат

При отображении страниц книги взаимодействуют 4 независимые системы координат:

| Пространство | Единицы | Описание | Источник |
| :--- | :--- | :--- | :--- |
| **PDF Point Space** | Пункты ($1\text{ pt} = \frac{1}{72}\text{ дюйма}$) | Физический размер страницы и элементов внутри документа. Не зависит от экрана и зума. | MuPDF (`page.getBounds()`, `toStructuredText()`, `search()`) |
| **GPUI Logical Pixels** | `Pixels` (`px`) | Логические пиксели компоновки интерфейса GPUI. Базовый масштаб $1.0$ при $100\%$ зуме соответствует $1\text{ pt} = 1\text{ px}$. | GPUI (`Bounds<Pixels>`, `size(px(w), px(h))`) |
| **Display Physical Pixels** | Физические пиксели экрана | Реальные аппаратные пиксели монитора (HiDPI / Retina / 4K). Получаются умножением логических пикселей на `scale_factor`. | Оконная подсистема ОС / `window.scale_factor()` |
| **Raster Pixmap Buffer** | Пиксели растрового изображения | Буфер пикселей PNG/BGRA, сформированный MuPDF при вызове рендера страницы с заданным разрешением `DPI`. | `mutool draw -r <DPI>` / `page.toPixmap()` |

---

## 2. Математические соотношения и формулы перевода

### 2.1. Логический размер страницы в интерфейсе (GPUI Layout)
Для отображения страницы при текущем коэффициенте масштабирования `zoom_factor` ($0.5$, $1.0$, $1.5$, $2.0$):
$$\text{width}_{\text{gpui}} = \text{page\_dims.width}_{\text{pt}} \times \text{zoom\_factor}$$
$$\text{height}_{\text{gpui}} = \text{page\_dims.height}_{\text{pt}} \times \text{zoom\_factor}$$

### 2.2. Масштабирование векторных оверлеев (поиск, ссылки, текст)
Все координаты bounding box (`BBox { x, y, w, h }`), возвращаемые MuPDF (поиск, ссылки, structured text), заданы в **PDF-пунктах** ($72\text{ DPI}$). 
Для их позиционирования поверх страницы в логических пикселях GPUI применяется прямое умножение:
$$x_{\text{gpui}} = x_{\text{pt}} \times \text{zoom\_factor}$$
$$y_{\text{gpui}} = y_{\text{pt}} \times \text{zoom\_factor}$$
$$w_{\text{gpui}} = w_{\text{pt}} \times \text{zoom\_factor}$$
$$h_{\text{gpui}} = h_{\text{pt}} \times \text{zoom\_factor}$$

Это преобразование реализовано в Rust методом `BBox::scaled(zoom_factor)`:
```rust
let scaled = bbox.scaled(zoom_factor);
let rect = Bounds::new(
    Point::new(bounds.origin.x + px(scaled.x), bounds.origin.y + px(scaled.y)),
    size(px(scaled.w), px(scaled.h)),
);
```

### 2.3. Расчёт целевого разрешения растрирования (Raster DPI)
Чтобы растровое изображение страницы не замыливалось при зуме и на HiDPI экранах, оно должно рендериться ровно в физические пиксели экрана:
$$\text{Target DPI} = 72 \times \text{zoom\_factor} \times \text{scale\_factor}$$

Примеры расчёта для монитора с `scale_factor = 1.0`:
- Зум $100\%$ (`zoom_factor = 1.0`): $\text{DPI} = 72 \times 1.0 \times 1.0 = 72\text{ DPI}$.
- Зум $150\%$ (`zoom_factor = 1.5`): $\text{DPI} = 72 \times 1.5 \times 1.0 = 108\text{ DPI}$.
- Зум $200\%$ (`zoom_factor = 2.0`): $\text{DPI} = 72 \times 2.0 \times 1.0 = 144\text{ DPI}$.

Примеры для HiDPI монитора (Retina / 4K) с `scale_factor = 2.0`:
- Зум $100\%$: $\text{DPI} = 72 \times 1.0 \times 2.0 = 144\text{ DPI}$.
- Зум $150\%$: $\text{DPI} = 72 \times 1.5 \times 2.0 = 216\text{ DPI}$.
- Зум $200\%$: $\text{DPI} = 72 \times 2.0 \times 2.0 = 288\text{ DPI}$.

---

## 3. Дефект текущей реализации в `libera-reader`

1. **Жёстко захардкоженный `PAGE_RENDER_DPI = 150`**:
   - В `crates/libera-reader/src/ui/pages/book-viewer/constants.rs`:
     ```rust
     pub const PAGE_RENDER_DPI: u32 = 150;
     ```
   - Загрузчик `loader.rs` всегда запрашивает рендер страницы с фиксированным разрешением $150\text{ DPI}$, игнорируя реальный масштаб `zoom_factor` и масштаб дисплея `window.scale_factor()`.
2. **Последствия**:
   - На масштабе $50\%$ в память загружается избыточно большая текстура ($150\text{ DPI}$ вместо требуемых $36\text{ DPI}$), потребляя память и время CPU.
   - На масштабе $200\%$ или на HiDPI экране $150\text{ DPI}$ недостаточно (требуется $200{-}300\text{ DPI}$), из-за чего растровая картинка растягивается видеокартой и текст становится мутным/размытым.
   - При этом векторный слой поиска (`PageTextLayer`) использует правильный `zoom_factor`, из-за чего возникает визуальный диссонанс: рамка подсветки чёткая, а буквы под ней — размытые.

---

## 4. Эталонное решение: Двухуровневый кэш растра (по образцу `mupdf.js`)

В веб-эталоне `other/mupdf.js/examples/simple-viewer/viewer.js`:
```javascript
// 1. Немедленное изменение CSS-размера контейнера (аппаратное масштабирование)
this.rootNode.style.width = (((this.size.width * this.zoom) / 72) | 0) + "px";
this.rootNode.style.height = (((this.size.height * this.zoom) / 72) | 0) + "px";

// 2. Отрисовка пикселей высокого разрешения с учетом devicePixelRatio
this.drawPromise = worker.drawPageAsPixmap(this.doc, this.pageNumber, zoom * devicePixelRatio);
```

### Архитектура для GPUI Kit:
1. **Фаза 1 (Мгновенный отклик при смене зума)**:
   - Контейнер `PageView` немедленно меняет размеры в логических пикселях: `px(width * zoom_factor)`.
   - Имеющаяся в кэше текстура `RenderImage` отрисовывается через `gpui::img().object_fit(Fill)` без задержки. Пользователь видит мгновенную реакцию интерфейса (60/120 FPS, без белых пятен).
2. **Фаза 2 (Асинхронный дорендер в нативном разрешении)**:
   - При смене зума формируется запрос `PageLoadRequest` с целевым DPI:
     ```rust
     let target_dpi = (72.0 * zoom_factor * window.scale_factor()).round() as u32;
     ```
   - Запрос проходит через дебаунс (~150 мс, чтобы не спамить процессами `mutool` при быстром кручении зума колесом мыши).
   - По завершении рендера новый `RenderImage` бесшовно заменяет старый в LRU-кэше, обеспечивая кристальную резкость шрифтов книги.
