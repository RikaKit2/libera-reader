# 04. Полнотекстовый поиск и навигация (Search Engine & Navigation)

Данный документ описывает алгоритм сквозного поиска по документу MuPDF, формат координат Quad, автомат состояний поиска и правила визуализации подсветки в GPUI Kit.

---

## 1. Поиск через C-движок MuPDF (`toStructuredText().search`)

Для поиска текста в PDF/EPUB документах MuPDF предоставляет специализированный API:
```javascript
var hits = page.toStructuredText().search(needle);
```

### Почему именно `toStructuredText().search`:
- Поиск выполняется с учётом внутренних CMap-таблиц и метрик встроенных шрифтов документа.
- Регистронезависимость (Case-insensitivity) и нормализация диакритических знаков встроены в ядро MuPDF.
- Движок возвращает массив совпадений, где каждое совпадение представляет собой массив полигонов **Quad** (четырёхугольников).

### Формат Quad-полигонов:
Каждый Quad состоит из 8 чисел `[ulx, uly, urx, ury, llx, lly, lrx, lry]`:
- `ul`: Upper Left (верхний левый угол)
- `ur`: Upper Right (верхний правый угол)
- `ll`: Lower Left (нижний левый угол)
- `lr`: Lower Right (нижний правый угол)

В скрипте поиска `crates/mutool/src/search.rs` Quad преобразуется в прямоугольник `BBox`:
```javascript
var x0 = Math.min(q[0], q[2], q[4], q[6]);
var y0 = Math.min(q[1], q[3], q[5], q[7]);
var x1 = Math.max(q[0], q[2], q[4], q[6]);
var y1 = Math.max(q[1], q[3], q[5], q[7]);
allHits.push({
    page: p + 1,
    bbox: { x: x0, y: y0, w: x1 - x0, h: y1 - y0 }
});
```
Координаты формируются в **PDF-пунктах** ($72\text{ DPI}$).

---

## 2. Скорость работы (Performance)

Сквозной поиск через `mutool run` с предкомпилированным скриптом по всей 586-страничной книге (`Всего шесть чисел...`) занимает **всего 0.8 секунды**.
Поэтому полнотекстовый поиск по всей книге не требует предварительной медленной индексации в SQLite или сторонних базах данных и выполняется по нажатию `Enter`.

---

## 3. Автомат состояний поиска (`BookViewerState`)

Для исключения ложных состояний интерфейса («Не найдено» во время набора текста) модель состояния разделяет фазы ввода и фазы подтверждённого поиска:

```rust
pub struct BookViewerState {
    /// Текущий введённый запрос в поле поиска
    pub search_query: String,
    /// Список найденных совпадений по всей книге
    pub search_results: Vec<DocumentSearchMatch>,
    /// Индекс текущего активного совпадения (0..search_results.len())
    pub current_search_idx: usize,
    /// Флаг: был ли фактически выполнен поиск по всей книге
    pub has_searched: bool,
    /// Флаг: выполняется ли асинхронный поиск в данный момент
    pub is_searching: bool,
}
```

### Диаграмма переходов состояний:
```
[Очищено] 
   │  (пользователь вводит символы)
   ▼
[Ввод запроса] ─── has_searched = false ───► Счётчик пуст (нет ложного "Не найдено")
   │  (нажатие Enter / клик "Найти")
   ▼
[Асинхронный поиск] ─── is_searching = true ───► Индикатор загрузки / спиннер
   │  (завершение mutool)
   ▼
[Результаты готовы] ─── has_searched = true
   ├── Если matches > 0  ──► "1 из 42", активное совпадение на экране
   └── Если matches == 0 ──► "Не найдено"
```

---

## 4. Навигация по совпадениям (Next / Previous)

Навигация циклическая (wrap-around):
- **Вперёд (`Enter` / стрелка `>` / `F3`)**:
  ```rust
  if !state.search_results.is_empty() {
      state.current_search_idx = (state.current_search_idx + 1) % state.search_results.len();
      let target_page = state.search_results[state.current_search_idx].page;
      state.scroll_to_page(target_page);
  }
  ```
- **Назад (`Shift+Enter` / стрелка `<` / `Shift+F3`)**:
  ```rust
  if !state.search_results.is_empty() {
      state.current_search_idx = (state.current_search_idx + state.search_results.len() - 1) % state.search_results.len();
      let target_page = state.search_results[state.current_search_idx].page;
      state.scroll_to_page(target_page);
  }
  ```

---

## 5. Отрисовка подсветки в `PageTextLayer`

Вся отрисовка подсветки сосредоточена в методе `paint` слоя `PageTextLayer`:

1. **Масштабирование**:
   ```rust
   let scaled = bbox.scaled(self.zoom_factor);
   let hit_bounds = Bounds::new(
       Point::new(bounds.origin.x + px(scaled.x), bounds.origin.y + px(scaled.y)),
       size(px(scaled.w), px(scaled.h)),
   );
   ```
2. **Динамические цвета темы (`cx.theme().primary`)**:
   - **Активное совпадение** (`is_active == true`):
     - Фон: `primary.opacity(0.65)`
     - Рамка: `primary` (100% непрозрачный контур, толщина `1px`, скругление `2px`)
   - **Пассивные совпадения на странице**:
     - Фон: `primary.opacity(0.20)`
     - Рамка: `primary.opacity(0.40)`
3. **Отрисовка через `window.paint_quad`**:
   Прямоугольники рисуются в аппаратном буфере окна GPUI, находясь под прозрачным текстом, что позволяет пользователю при необходимости выделить подсвеченный фрагмент мышью и скопировать его.
