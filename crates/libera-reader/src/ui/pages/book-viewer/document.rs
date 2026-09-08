use crate::db::models::books::book::BookPath;
use mutool::{
  MuToolError, OutlineNode, PageDimensions, get_document_outline, get_document_page_info,
};

#[derive(Debug, Clone)]
pub struct DocumentData {
  pub book_path: BookPath,
  pub total_pages: usize,
  pub page_sizes: Vec<PageDimensions>,
  pub outline: Vec<OutlineNode>,
}

impl DocumentData {
  pub fn load(path: BookPath) -> Result<Self, MuToolError> {
    let fs_path = path.as_pathbuf();
    let page_info = get_document_page_info(&fs_path)?;
    let outline = get_document_outline(&fs_path).unwrap_or_default();

    Ok(Self {
      book_path: path,
      total_pages: page_info.total_pages,
      page_sizes: page_info.pages,
      outline,
    })
  }

  pub fn page_size(&self, page: usize) -> PageDimensions {
    if page == 0 || page > self.total_pages {
      PageDimensions::default()
    } else {
      self.page_sizes.get(page - 1).copied().unwrap_or_default()
    }
  }
}
