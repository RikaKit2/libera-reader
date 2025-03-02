#include "wrapper.h"
#include <mupdf/fitz/buffer.h>
#include <mupdf/fitz/color.h>
#include <mupdf/fitz/context.h>
#include <mupdf/fitz/document.h>
#include <mupdf/fitz/output.h>
#include <mupdf/fitz/pixmap.h>
#include <mupdf/fitz/store.h>
#include <mupdf/fitz/write-pixmap.h>
#include <stdbool.h>
#include <stdint.h>

typedef struct {
  bool flag;
  const char *err_msg;
} mupdf_status;

typedef struct {
  mupdf_status status;
  fz_context *ctx;
} mupdf_ctx;

typedef struct {
  mupdf_status status;
  fz_document *doc;
} mupdf_doc;

typedef struct {
  mupdf_status status;
  int count;
} mupdf_page_count;

typedef struct {
  mupdf_status status;
  fz_pixmap *pix;
} mupdf_pixmap;

typedef struct {
  mupdf_status status;
  fz_page *page;
} mupdf_page;

typedef struct {
  mupdf_status status;
  const char *text;
} mupdf_stext_json;

typedef struct {
  mupdf_status status;
  const char *text;
} mupdf_pix_as_jpeg;

typedef struct {
  mupdf_status status;
  const char *text;
} mupdf_text_from_page;

typedef struct {
  mupdf_status status;
  fz_outline *outline;
} mupdf_outline;

typedef struct {
  mupdf_status status;
  char *metadata;
} mupdf_metadata;

void set_err_in_poss_ctx(mupdf_ctx *res, const char *msg, fz_context *ctx) {
  res->status.flag = false;
  res->status.err_msg = msg;
  fz_drop_context(ctx);
}

void write_error(fz_context *ctx, mupdf_status *to) {
  to->flag = false;
  to->err_msg = fz_caught_message(ctx);
}
/*
  max_store: Maximum size in bytes of the resource store, before
  it will start evicting cached resources such as fonts and
  images. FZ_STORE_UNLIMITED can be used if a hard limit is not
  desired. Use FZ_STORE_DEFAULT to get a reasonable size.
  FZ_STORE_UNLIMITED = 0,
  FZ_STORE_DEFAULT = 256 << 20 = 268435456 = 268.435456 Megabyte,
*/
mupdf_ctx mupdf_new_context(const size_t max_store) {
  mupdf_ctx output;
  fz_context *ctx = fz_new_context(NULL, NULL, max_store);
  if (ctx == NULL) {
    set_err_in_poss_ctx(&output, "ctx is null", ctx);
  } else {
    fz_try(ctx) {
      fz_register_document_handlers(ctx);
      output.status.flag = true;
      output.ctx = ctx;
    }
    fz_catch(ctx) { set_err_in_poss_ctx(&output, fz_caught_message(ctx), ctx); }
  };
  return output;
}

/* Document */
mupdf_doc mupdf_open_document(fz_context *ctx, const char *path_to_doc) {
  mupdf_doc output;
  fz_document *doc = NULL;
  fz_var(doc);
  fz_try(ctx) {
    doc = fz_open_document(ctx, path_to_doc);
    output.status.flag = true;
    output.doc = doc;
  }
  fz_catch(ctx) {
    fz_drop_document(ctx, doc);
    write_error(ctx, &output.status);
  };
  return output;
}

mupdf_page_count mupdf_doc_page_count(fz_context *ctx, fz_document *doc) {
  mupdf_page_count output;
  fz_try(ctx) {
    output.status.flag = true;
    output.count = fz_count_pages(ctx, doc);
  }
  fz_catch(ctx) { write_error(ctx, &output.status); };
  return output;
};

mupdf_outline mupdf_load_outline(fz_context *ctx, fz_document *doc) {
  mupdf_outline output;
  fz_try(ctx) {
    output.outline = fz_load_outline(ctx, doc);
    output.status.flag = true;
  }
  fz_catch(ctx) { write_error(ctx, &output.status); }
  return output;
}

mupdf_metadata mupdf_lookup_metadata(fz_context *ctx, fz_document *doc,
                                     const char *key) {
  mupdf_metadata output;
  static char buf[500];
  fz_try(ctx) {
    if (fz_lookup_metadata(ctx, doc, key, buf, sizeof buf) > 0) {
      output.status.flag = true;
      output.metadata = buf;
    } else {
      output.status.flag = false;
      output.status.err_msg = "key is not recognized or found";
    }
  }
  fz_catch(ctx) { write_error(ctx, &output.status); }
  return output;
}

/* Page */
mupdf_page mupdf_load_page(fz_context *ctx, fz_document *doc,
                           const int page_num) {
  mupdf_page output;
  fz_page *page = NULL;
  fz_var(page);
  fz_try(ctx) {
    page = fz_load_page(ctx, doc, page_num);
    output.status.flag = true;
    output.page = page;
  }
  fz_catch(ctx) {
    fz_drop_page(ctx, page);
    write_error(ctx, &output.status);
  }
  return output;
}

mupdf_pixmap mupdf_page_to_pixmap(fz_context *ctx, fz_page *page,
                                  const float alpha, const float zoom) {
  mupdf_pixmap output;
  fz_pixmap *pixmap = NULL;
  fz_var(pixmap);
  fz_try(ctx) {
    const fz_matrix ctm = fz_scale(zoom, zoom);
    pixmap = fz_new_pixmap_from_page(ctx, page, ctm, fz_device_rgb(ctx), alpha);
    output.status.flag = true;
    output.pix = pixmap;
  }
  fz_catch(ctx) {
    fz_drop_pixmap(ctx, pixmap);
    write_error(ctx, &output.status);
  };
  return output;
}

mupdf_stext_json mupdf_stext_page_as_json_from_page(fz_context *ctx,
                                                    fz_page *page,
                                                    const float scale) {
  mupdf_stext_json output;
  fz_buffer *buf = NULL;
  fz_output *out = NULL;
  fz_stext_page *stext_page = NULL;
  fz_var(buf);
  fz_var(out);
  fz_var(stext_page);

  fz_try(ctx) {
    buf = fz_new_buffer(ctx, 8192);
    out = fz_new_output_with_buffer(ctx, buf);
    stext_page = fz_new_stext_page_from_page(ctx, page, NULL);
    fz_print_stext_page_as_json(ctx, out, stext_page, scale);

    fz_close_output(ctx, out);
    output.status.flag = true;
    output.text = fz_string_from_buffer(ctx, buf);
  }
  fz_always(ctx) {
    fz_drop_output(ctx, out);
    fz_terminate_buffer(ctx, buf);
    fz_drop_buffer(ctx, buf);
    fz_drop_stext_page(ctx, stext_page);
  }
  fz_catch(ctx) { write_error(ctx, &output.status); }
  return output;
}

mupdf_text_from_page mupdf_page_as_plain_text(fz_context *ctx, fz_page *page) {
  mupdf_text_from_page output;
  fz_buffer *buf = NULL;
  fz_output *out = NULL;
  fz_stext_page *text = NULL;
  fz_var(text);
  fz_var(buf);
  fz_var(out);
  fz_try(ctx) {
    text = fz_new_stext_page_from_page(ctx, page, NULL);
    buf = fz_new_buffer(ctx, 8192);
    out = fz_new_output_with_buffer(ctx, buf);
    fz_print_stext_page_as_text(ctx, out, text);
    fz_close_output(ctx, out);
  }
  fz_always(ctx) {
    fz_drop_output(ctx, out);
    fz_drop_stext_page(ctx, text);
    output.status.flag = true;
    output.text = fz_string_from_buffer(ctx, buf);
  }
  fz_catch(ctx) { write_error(ctx, &output.status); }
  return output;
}

/* Pixmap */
mupdf_status mupdf_get_pixmap_as_jpeg(fz_context *ctx, fz_pixmap *pix,
                                      const int quality,
                                      unsigned char *out_storage) {
  mupdf_status output;
  unsigned char *datap;
  fz_try(ctx) {
    fz_buffer *buf = fz_new_buffer_from_pixmap_as_jpeg(
        ctx, pix, fz_default_color_params, quality, 0);

    unsigned char *datap;
    size_t len = fz_buffer_storage(ctx, buf, &datap);
    memcpy(out_storage, &datap, len);
    output.flag = true;
  }
  fz_catch(ctx) { write_error(ctx, &output); }
  return output;
}

/* Other */
void write_error_to_storage(mupdf_status *source, mupdf_status *dest) {
  dest->flag = source->flag;
  dest->err_msg = source->err_msg;
}

mupdf_status mupdf_get_thumbnail_from_document(
    const size_t max_store, const char *path_to_doc, const int page_num,
    const float alpha, const float zoom, const int quality,
    unsigned char *out_storage) {
  mupdf_status output;
  mupdf_ctx ctx_res = mupdf_new_context(max_store);
  if (ctx_res.status.flag) {
    fz_context *ctx = ctx_res.ctx;
    mupdf_doc doc_res = mupdf_open_document(ctx, path_to_doc);
    if (doc_res.status.flag) {
      fz_document *doc = doc_res.doc;
      mupdf_page page_res = mupdf_load_page(ctx, doc, page_num);
      if (page_res.status.flag) {
        fz_page *page = page_res.page;
        mupdf_pixmap pixmap_res = mupdf_page_to_pixmap(ctx, page, alpha, zoom);
        if (pixmap_res.status.flag) {
          fz_pixmap *pix = pixmap_res.pix;
          mupdf_status extracting_status =
              mupdf_get_pixmap_as_jpeg(ctx, pix, quality, out_storage);
          if (!extracting_status.flag) {
            write_error_to_storage(&extracting_status, &output);
          }
        } else {
          write_error_to_storage(&pixmap_res.status, &output);
        }
      } else {
        write_error_to_storage(&page_res.status, &output);
      }

    } else {
      write_error_to_storage(&doc_res.status, &output);
    }

  } else {
    write_error_to_storage(&ctx_res.status, &output);
  }
  return output;
}

mupdf_status
mupdf_get_thumbnail_from_document2(const size_t max_store, const char *doc_path,
                                   const int page_num, const float alpha,
                                   const float zoom, const int quality,
//                                   unsigned char *out_storage
                                   const char *path_to_out) {
  mupdf_status output;
  float rotate;
  int page_count;
  fz_context *ctx;
  fz_document *doc;
  fz_pixmap *pix;
  fz_matrix ctm;

  /* Create a context to hold the exception stack and various caches. */
  ctx = fz_new_context(NULL, NULL, FZ_STORE_UNLIMITED);
  if (!ctx) {
    output.err_msg = "cannot create mupdf context";
    output.flag = false;
  }

  /* Register the default file types to handle. */
  fz_try(ctx) fz_register_document_handlers(ctx);
  fz_catch(ctx) {
    fz_report_error(ctx);
    output.err_msg = "cannot register document handlers";
    output.flag = false;
  }

  /* Open the document. */
  fz_try(ctx) doc = fz_open_document(ctx, doc_path);
  fz_catch(ctx) {
    fz_report_error(ctx);
    output.err_msg = "cannot open document";
    output.flag = false;
  }

  /* Count the number of pages. */
  fz_try(ctx) page_count = fz_count_pages(ctx, doc);
  fz_catch(ctx) {
    fz_report_error(ctx);
    output.err_msg = "cannot count number of pages";
    output.flag = false;
  }

  if (page_num < 0 || page_num >= page_count) {
    output.err_msg = "page number out of range";
    output.flag = false;
  }
  ctm = fz_scale(zoom / 100, zoom / 100);
  ctm = fz_pre_rotate(ctm, rotate);

  /* Render page to an RGB pixmap. */
  fz_try(ctx) pix = fz_new_pixmap_from_page_number(ctx, doc, page_num, ctm,
                                                   fz_device_rgb(ctx), alpha);
  fz_catch(ctx) {
    fz_report_error(ctx);
    output.err_msg = "cannot render page";
    output.flag = false;
  }
//  fz_buffer *buf;
//  fz_try(ctx) {
//    buf = fz_new_buffer_from_pixmap_as_jpeg(ctx, pix, fz_default_color_params,
//                                            quality, 0);
//  }
//  fz_catch(ctx) {
//    fz_report_error(ctx);
//    output.err_msg = "cannot extract jpeg from pixmap";
//    output.flag = false;
//    fz_drop_buffer(ctx, buf);
//  }
  fz_try(ctx) {
    fz_save_pixmap_as_jpeg(ctx, pix, path_to_out, quality);
    output.flag = true;
  }
  fz_catch(ctx) {
    output.flag = false;
    output.err_msg = fz_caught_message(ctx);
  }
//  unsigned char *datap;
//  size_t len = fz_buffer_storage(ctx, buf, &datap);
//  memcpy(out_storage, &datap, len);

  /* Clean up. */
  fz_drop_pixmap(ctx, pix);
  fz_drop_document(ctx, doc);
  fz_drop_context(ctx);
  return output;
}

mupdf_status mupdf_save_pixmap_as_jpeg(fz_context *ctx, fz_pixmap *pix,
                                    const int quality,
                                    const char *path_to_out) {
  mupdf_status output;
  fz_try(ctx) {
    fz_save_pixmap_as_jpeg(ctx, pix, path_to_out, quality);
    output.flag = true;
  }
  fz_catch(ctx) {
    output.flag = false;
    output.err_msg = fz_caught_message(ctx);
  }
  return output;
}
