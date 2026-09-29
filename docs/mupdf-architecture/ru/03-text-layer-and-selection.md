# 03. Текстовый слой и выделение мышью (Text Layer & Selection)

Данный документ описывает структуру данных структурированного текста MuPDF, природу шрифтового дрейфа (Font Metric Mismatch) и архитектуру точного выделения текста в GPUI Kit.

---

## 1. Структура данных `PageStructuredText`

MuPDF извлекает текст со страницы с сохранением геометрии через метод:
```javascript
page.toStructuredText("preserve-spans").asJSON()
```
В Rust эта структура представлена в `crates/mutool/src/stext.rs`:

```rust
pub struct PageStructuredText {
    pub blocks: Vec<TextBlock>,
}

pub struct TextBlock {
    pub block_type: String, // "text" или "image"
    pub bbox: BBox,         // { x, y, w, h } в PDF-пунктах (72 DPI)
    pub lines: Vec<TextLine>,
}

pub struct TextLine {
    pub bbox: BBox,
    pub font: Option<FontInfo>, // name, family, weight, style, size
    pub text: String,           // Текст строки в UTF-8
    pub x: f32,
    pub y: f32,
    pub wmode: i32,             // 0 = горизонтальный текст, 1 = вертикальный
}
```

Каждая строка `TextLine` содержит:
- Точный `bbox` строки, рассчитанный самим движком MuPDF на основе встроенного в PDF шрифта.
- Базовую линию текста `y` и точку начала `x`.
- Размер шрифта `font.size` в PDF-пунктах.

---

## 2. Физика бага: Шрифтовой дрейф (Font Metric Mismatch)

### Почему возникает расхождение:
1. **Встроенный шрифт PDF**: При верстке книги текст набирается издательским шрифтом (например, кастомный сабсет `BCDHEE+___WRD_EMBED_SUB_43` или `Minion Pro`). Файл шрифта с уникальными апрошами (межзнаковыми интервалами) и кернингом внедрён прямо в бинарник PDF.
2. **Системный шрифт интерфейса**: При шейпинге текста в GPUI (`window.text_style()`) используется шрифт системы (например, `Inter`, `Roboto` или системный sans-serif).
3. **Разница ширин символов**:
   - Буква «В» в шрифте книги имеет ширину $10.2\text{ pt}$, а в `Inter` — $12.8\text{ pt}$.
   - Буква «ы» в шрифте книги — $12.1\text{ pt}$, в `Inter` — $14.5\text{ pt}$.
   - Пробел в шрифте книги — $3.5\text{ pt}$, в `Inter` — $4.2\text{ pt}$.
4. **Результат**:
   Фраза из 2–3 слов («Вы смогли») в шрифте книги имеет ширину $61.46\text{ pt}$ ($92.19\text{ px}$ при зуме 150%). Тот же текст, свёрстанный через системный `TextLayout`, занимает $76.93\text{ pt}$ ($115.4\text{ px}$).
   Накопленная дельта в $+23.2\text{ px}$ приводила к тому, что прямоугольник подсветки съезжал вправо, накрывая пробел и первую букву соседнего слова.

---

## 3. Решение в оригинальном веб-эталоне (`mupdf.js`)

В веб-браузере невозможно загрузить кастомный сабсет шрифта из PDF в системный стек стилей CSS напрямую для каждого абзаца. 
Поэтому Artifex в `simple-viewer/viewer.js` применил архитектуру SVG-подгонки:

```javascript
let text = document.createElementNS("http://www.w3.org/2000/svg", "text");
text.setAttribute("x", line.bbox.x * scale + "px");
text.setAttribute("y", line.y * scale + "px");
text.style.fontSize = line.font.size * scale + "px";
text.setAttribute("textLength", line.bbox.w * scale + "px");
text.setAttribute("lengthAdjust", "spacingAndGlyphs");
text.textContent = line.text;
```

### Как это работает:
- Браузерный атрибут `textLength = line.bbox.w * scale` принудительно задаёт ширину строки равной точной ширине из MuPDF.
- `lengthAdjust="spacingAndGlyphs"` указывает рендереру масштабировать межбуквенные интервалы и ширину глифов так, чтобы строка **физически заняла ровно отведенный отрезок от `line.bbox.x` до `line.bbox.x + line.bbox.w`**.
- В результате прозрачный текст ложится пиксель-в-пиксель поверх растровых букв страницы, и выделение мыши совпадает с картинкой.

---

## 4. Реализация в GPUI Kit

В GPUI Kit механизм SVG отсутствует, поэтому интеграция строится через подсистему `gpui_base::TextSelection`:

### 4.1. Формирование строк `PageTextLineData`
Для каждой строки `TextLine`:
```rust
let scaled = line.bbox.scaled(zoom_factor);
let font_size = line.font.as_ref().map_or(scaled.h * 0.8, |f| f.size * zoom_factor);

let mut line_style = text_style.clone();
line_style.font_size = px(font_size).into();
line_style.line_height = px(scaled.h).into();
line_style.color = gpui::transparent_black(); // Полностью прозрачный цвет

let runs = vec![line_style.to_run(line.text.len())];
let shared_text = SharedString::from(line.text.clone());
let styled_text = StyledText::new(shared_text.clone()).with_runs(runs);

let line_bounds = Bounds::new(
    Point::new(px(scaled.x), px(scaled.y)),
    size(px(scaled.w), px(scaled.h)),
);
```

### 4.2. Точный расчёт границ поддиапазонов (`compute_line_selection_bounds`)
При частичном выделении строки мышью:
- `layout.position_for_index(start)` и `layout.position_for_index(end)` используются для нахождения относительных смещений внутри строки.
- Границы принудительно ограничиваются (`clamped`) физическим прямоугольником строки:
  ```rust
  let left_bound = line_window_bounds.origin.x;
  let right_bound = line_window_bounds.right();
  let x_start = layout.position_for_index(range.start)
      .map(|p| p.x.max(left_bound).min(right_bound))
      .unwrap_or(left_bound);
  ```
- Для полных совпадений поиска (из `search_document_text`) вычисления через `layout` **полностью исключены**: координаты берутся напрямую из MuPDF `hit.bbox.scaled(zoom_factor)`.

### 4.3. Регистрация в диспетчере выделения GPUI
В фазе `prepaint`:
```rust
self.selection_handle.register(
    TextSelectionRegistration::new(page_hitbox.clone(), bounds)
        .with_document_order(self.page_number as u64)
        .with_text_bounds(text_bounds),
    window,
    cx,
);
```
В фазе `paint`:
```rust
let projection = self.selection_handle.update_runs(&runs, cx);
```
Это обеспечивает нативное поведение:
- Выделение мышью через границы строк и страниц.
- Копирование в буфер обмена (`Ctrl+C` / `Cmd+C`) чистого текста книги в кодировке UTF-8.
- Подсветка выделения полупрозрачным цветом темы: `cx.theme().primary.opacity(0.35)`.
