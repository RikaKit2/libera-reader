# 06. Annotations and Text Markups

This document describes the MuPDF annotation subsystem, supported annotation types, the data model from WebViewer (`types.d.ts`), rendering strategies, and PDF baking.

---

## 1. Supported Annotation Types in MuPDF

MuPDF implements the PDF ISO 32000 annotation standards:

| Type | Description | Geometry | Primary Use Case |
| :--- | :--- | :--- | :--- |
| **`Highlight`** | Semi-transparent highlight marker | `QuadPoints` (array of quad points) | Emphasizing key phrases or quotes |
| **`Underline`** | Solid horizontal text underline | `QuadPoints` | Highlighting definitions or terms |
| **`StrikeOut`** | Text strikethrough | `QuadPoints` | Marking deprecated or deleted text |
| **`Squiggly`** | Wavy text underline | `QuadPoints` | Marking typos or review notes |
| **`Text`** | Sticky note icon | `Rect [x, y, w, h]` + `contents: String` | Margin comments |
| **`FreeText`** | Direct on-page visible text box | `Rect` + `font`, `fontSize`, `color` | Inline textual additions |
| **`Stamp`** | Rubber stamp (Approved, Draft, custom) | `Rect` + vector/raster graphic | Document review approvals |
| **`Redact`** | Content redaction box | `QuadPoints` / `Rect` | Blacking out confidential text |

---

## 2. WebViewer Data Model (`types.d.ts`)

In `other/mupdf-webviewer/types.d.ts`, annotations are structured as:

```typescript
export interface Annotation {
    oid: number;                 // Unique PDF object ID
    type: AnnotType;             // HIGHLIGHT, UNDERLINE, STRIKE, TEXT, FREETEXT, etc.
    pageIndex: number;           // 0-indexed page number
    rect: [number, number, number, number]; // [x0, y0, x1, y1]
    quadPoints?: number[];       // Glyph coverage polygons for text markups
    strokeColor?: string;        // Hex/RGB border color
    fillColor?: string;          // Hex/RGB fill color
    opacity?: number;            // 0.0 .. 1.0
    contents?: string;           // Note text content
    author?: string;             // Creator name
    modificationDate?: string;   // Timestamp
}
```

### Lifecycle Events:
- `ANNOTATION_CREATE`: Emitted when the user completes a new annotation.
- `ANNOTATION_MODIFY`: Color, comment, or geometry updates.
- `ANNOTATION_REMOVE`: Annotation deletion.
- `ANNOTATION_SELECTION_CHANGE`: Focus shifted to an annotation for property editing (`ANNOTATION_PROPERTY_PANEL`).

---

## 3. Creating Annotations in MuPDF Core

From `other/mupdf.js/docs/how-to-guide/annotations/`:
```javascript
// 1. Create a highlight annotation
let annotation = page.createAnnotation("Highlight");
annotation.setColor([1, 0.8, 0]); // RGB yellow
annotation.setQuadPoints([
    [x0, y0, x1, y1, x2, y2, x3, y3] // Quad coordinates of target text
]);
annotation.update();

// 2. Bake annotations permanently into the PDF
let updatedPdfBytes = document.saveToBuffer("incremental").asUint8Array();
```

---

## 4. Integration Roadmap for GPUI Kit (`libera-reader`)

Implementation is divided into two sequential milestones:

### Milestone 1: Read-Only Annotation Display
1. `mutool draw` automatically includes standard document annotations when generating the page pixmap.
2. Structured extraction parses interactive notes and links via `page.getAnnotations()`.
3. An overlay layer renders interactive note pins and tooltips (`Tooltip`).
4. A dedicated sidebar tab `SidebarTab::Annotations` (`Panel::ANNOTATION`) aggregates all document annotations into a chronological list with jump-on-click navigation.

### Milestone 2: Interactive Annotation Creation
1. **Contextual Action Menu**:
   Releasing mouse drag selection (`TextSelection`) presents a floating action popup (`CONTEXT_MENU_HIGHLIGHT`) with:
   - "Highlight" (with palette selection)
   - "Underline"
   - "Strikeout"
   - "Copy Quote"
2. **Persistence**:
   - Annotations are saved in the reader's local database (`native_db`) keyed by document hash and page indices.
   - An optional "Bake & Export" command executes `mutool run` to write modifications directly back to the `.pdf` file.
