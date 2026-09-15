# 06. Аннотации и маркеры (Annotations)

Данный документ описывает систему аннотаций MuPDF, их типы, структуру данных в WebViewer (`types.d.ts`), способы отображения и запекания в PDF.

---

## 1. Типы аннотаций в MuPDF

MuPDF поддерживает полный стандарт аннотаций PDF (ISO 32000):

| Тип | Описание | Геометрия | Применение |
| :--- | :--- | :--- | :--- |
| **`Highlight`** | Полупрозрачное выделение маркером | `QuadPoints` (массив четвёрок точек) | Выделение ключевых цитат текста |
| **`Underline`** | Сплошное подчёркивание текста | `QuadPoints` | Подчёркивание важных терминов |
| **`StrikeOut`** | Зачёркивание текста | `QuadPoints` | Пометка удалённого/устаревшего текста |
| **`Squiggly`** | Волнистое подчёркивание | `QuadPoints` | Пометка ошибок, примечаний |
| **`Text`** | Значок заметки (Note icon) | `Rect [x, y, w, h]` + `contents: String` | Комментарии на полях |
| **`FreeText`** | Свободно видимый блок текста на листе | `Rect` + `font`, `fontSize`, `color` | Текстовые заметки прямо на странице |
| **`Stamp`** | Штамп (Approved, Draft, кастомный) | `Rect` + изображение/вектор | Штампы согласования |
| **`Redact`** | Область цензурирования/скрытия | `QuadPoints` / `Rect` | Удаление конфиденциальных данных |

---

## 2. Модель данных в `mupdf-webviewer` (`types.d.ts`)

В эталонном SDK `other/mupdf-webviewer/types.d.ts` аннотации строго типизированы:

```typescript
export interface Annotation {
    oid: number;                 // Уникальный ID объекта в PDF
    type: AnnotType;             // HIGHLIGHT, UNDERLINE, STRIKE, TEXT, FREETEXT, etc.
    pageIndex: number;           // 0-indexed страница
    rect: [number, number, number, number]; // [x0, y0, x1, y1]
    quadPoints?: number[];       // Полигоны охвата букв для текстовых аннотаций
    strokeColor?: string;        // Hex/RGB цвет контура
    fillColor?: string;          // Hex/RGB цвет заливки
    opacity?: number;            // 0.0 .. 1.0
    contents?: string;           // Текстовое содержимое заметки
    author?: string;             // Имя автора
    modificationDate?: string;   // Дата создания/правки
}
```

### События жизненного цикла аннотаций:
- `ANNOTATION_CREATE`: создание новой аннотации при выделении текста пользователем.
- `ANNOTATION_MODIFY`: изменение цвета, комментария или границ.
- `ANNOTATION_REMOVE`: удаление аннотации.
- `ANNOTATION_SELECTION_CHANGE`: выбор аннотации курсором для отображения панели свойств (`ANNOTATION_PROPERTY_PANEL`).

---

## 3. Эталон создания аннотаций в MuPDF C/JS Core

В `other/mupdf.js/docs/how-to-guide/annotations/`:
```javascript
// 1. Создание текстового маркера (Highlight)
let annotation = page.createAnnotation("Highlight");
annotation.setColor([1, 0.8, 0]); // RGB жёлтый
annotation.setQuadPoints([
    [x0, y0, x1, y1, x2, y2, x3, y3] // Координаты выделенных слов
]);
annotation.update();

// 2. Запекание аннотаций в PDF (Save incremental)
let updatedPdfBytes = document.saveToBuffer("incremental").asUint8Array();
```

---

## 4. Архитектура интеграции в GPUI Kit (`libera-reader`)

Для `libera-reader` внедрение аннотаций делится на два этапа:

### Этап 1: Чтение и рендеринг существующих аннотаций (Read-only)
1. При рендеринге страницы `mutool draw` автоматически включает отображение существующих векторных аннотаций в растре книги.
2. При загрузке структурированного текста через `mutool` извлекается список интерактивных аннотаций (`page.getAnnotations()`).
3. В `PageView` добавляется слой `PageAnnotationsLayer` (или объединяется с `PageLinksLayer`), отображающий иконки заметок и всплывающие подсказки (`Tooltip`).
4. В боковой панели активируется вкладка `SidebarTab::Annotations` (`Panel::ANNOTATION`), где отображается единый хронологический список всех пометок в книге с быстрым переходом по клику.

### Этап 2: Создание пользовательских аннотаций (Interactive)
1. **Контекстное меню при выделении текста**:
   При отпускании мыши после выделения диапазона символов (`TextSelection`) появляется контекстный тулбар / меню (`CONTEXT_MENU_HIGHLIGHT`) с кнопками:
   - «Выделить маркером» (Highlight с выбором цвета)
   - «Подчеркнуть» (Underline)
   - «Зачеркнуть» (StrikeOut)
   - «Скопировать цитату» (Copy)
2. **Хранение**:
   - Локальные заметки сохраняются в базе данных `native_db` читалки, привязываясь к хешу книги и номерам страниц.
   - По желанию пользователя (пункт меню «Экспорт с аннотациями» / «Запечь в PDF») вызывается `mutool run` для сохранения физических аннотаций прямо в файл `.pdf`.
