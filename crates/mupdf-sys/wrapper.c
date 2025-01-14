#include "wrapper.h"
#include <mupdf/fitz/buffer.h>
#include <mupdf/fitz/color.h>
#include <mupdf/fitz/context.h>
#include <mupdf/fitz/document.h>
#include <mupdf/fitz/pixmap.h>
#include <mupdf/fitz/store.h>
#include <mupdf/fitz/write-pixmap.h>
#include <stdbool.h>


typedef struct {
  bool status;
  const char *err_msg;
} mupdf_res;

typedef struct {
  bool status;

  union {
    fz_context *ctx;
    const char *err_msg;
  } value;
} mupdf_ctx;

typedef struct {
  bool status;

  union {
    fz_document *doc;
    const char *err_msg;
  } value;
} mupdf_doc;

typedef struct {
  bool status;

  union {
    int count;
    const char *err_msg;
  } value;
} mupdf_page_count;

typedef struct {
  bool status;

  union {
    fz_pixmap *pix;
    const char *err_msg;
  } value;
} mupdf_pixmap;

typedef struct {
  bool status;

  union {
    fz_page *page;
    const char *err_msg;
  } value;
} mupdf_page;

typedef struct {
  bool status;

  union {
    const char *text;
    const char *err_msg;
  } value;
} mupdf_stext_json;

typedef struct {
  bool status;

  union {
    struct {
      unsigned char *data;
      size_t data_len;
    } value;

    const char *err_msg;
  } inner;
} mupdf_pix_as_jpeg;

typedef struct {
  bool status;

  union {
    const char *text;
    const char *err_msg;
  } value;
} mupdf_text_from_page;

typedef struct {
  bool status;

  union {
    fz_outline *res;
    const char *err_msg;
  } value;
} mupdf_outline;

typedef struct {
  bool status;

  union {
    char *res;
    const char *err_msg;
  } value;
} mupdf_metadata;

void set_err_in_poss_ctx(mupdf_ctx *res, const char *msg, fz_context *ctx) {
  res->status = false;
  res->value.err_msg = msg;
  fz_drop_context(ctx);
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
  mupdf_ctx res;
  fz_context *ctx = fz_new_context(NULL, NULL, max_store);
  if (ctx == NULL) {
    set_err_in_poss_ctx(&res, "ctx is null", ctx);
  } else {
    fz_try(ctx) {
      fz_register_document_handlers(ctx);
      res.status = true;
      res.value.ctx = ctx;
    }
    fz_catch(ctx) { set_err_in_poss_ctx(&res, fz_caught_message(ctx), ctx); }
  };
  return res;
}

/* Document */
mupdf_doc mupdf_open_document(fz_context *ctx, const char *path_to_doc) {
  mupdf_doc res;
  fz_document *doc = NULL;
  fz_var(doc);
  fz_try(ctx) {
    doc = fz_open_document(ctx, path_to_doc);
    res.status = true;
    res.value.doc = doc;
  }
  fz_catch(ctx) {
    fz_drop_document(ctx, doc);
    res.status = false;
    res.value.err_msg = fz_caught_message(ctx);
  };
  return res;
}

mupdf_page_count mupdf_doc_page_count(fz_context *ctx, fz_document *doc) {
  mupdf_page_count res;
  fz_try(ctx) {
    res.status = true;
    res.value.count = fz_count_pages(ctx, doc);
  }
  fz_catch(ctx) {
    res.status = false;
    res.value.err_msg = fz_caught_message(ctx);
  };
  return res;
};

mupdf_outline mupdf_load_outline(fz_context *ctx, fz_document *doc) {
  mupdf_outline res;
  fz_try(ctx) {
    res.value.res = fz_load_outline(ctx, doc);
    res.status = true;
  }
  fz_catch(ctx) {
    res.status = false;
    res.value.err_msg = fz_caught_message(ctx);
  }
  return res;
}

mupdf_metadata mupdf_lookup_metadata(fz_context *ctx, fz_document *doc,
                                     const char *key) {
  mupdf_metadata res;
  static char buf[500];
  fz_try(ctx) {
    if (fz_lookup_metadata(ctx, doc, key, buf, sizeof buf) > 0) {
      res.status = true;
      res.value.res = buf;
    } else {
      res.status = false;
      res.value.err_msg = "key is not recognized or found";
    }
  }
  fz_catch(ctx) {
    res.status = false;
    res.value.err_msg = fz_caught_message(ctx);
  }
  return res;
}

/* Page */
mupdf_page mupdf_load_page(fz_context *ctx, fz_document *doc,
                           const int page_num) {
  mupdf_page res;
  fz_page *page = NULL;
  fz_var(page);
  fz_try(ctx) {
    page = fz_load_page(ctx, doc, page_num);
    res.status = true;
    res.value.page = page;
  }
  fz_catch(ctx) {
    fz_drop_page(ctx, page);
    res.status = false;
    res.value.err_msg = fz_caught_message(ctx);
  }
  return res;
}

mupdf_pixmap mupdf_page_to_pixmap(fz_context *ctx, fz_page *page,
                                  const float alpha, const float zoom) {
  mupdf_pixmap res;
  fz_pixmap *pixmap = NULL;
  fz_var(pixmap);
  fz_try(ctx) {
    const fz_matrix ctm = fz_scale(zoom, zoom);
    pixmap = fz_new_pixmap_from_page(ctx, page, ctm, fz_device_rgb(ctx), alpha);
    res.status = true;
    res.value.pix = pixmap;
  }
  fz_catch(ctx) {
    fz_drop_pixmap(ctx, pixmap);
    res.status = false;
    res.value.err_msg = fz_caught_message(ctx);
  };
  return res;
}

mupdf_stext_json mupdf_stext_page_as_json_from_page(fz_context *ctx,
                                                    fz_page *page,
                                                    const float scale) {
  mupdf_stext_json res;
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
    res.status = true;
    res.value.text = fz_string_from_buffer(ctx, buf);
  }
  fz_always(ctx) {
    fz_drop_output(ctx, out);
    fz_terminate_buffer(ctx, buf);
    fz_drop_buffer(ctx, buf);
    fz_drop_stext_page(ctx, stext_page);
  }
  fz_catch(ctx) {
    res.status = false;
    res.value.err_msg = fz_caught_message(ctx);
  }
  return res;
}

mupdf_text_from_page mupdf_page_as_plain_text(fz_context *ctx, fz_page *page) {
  mupdf_text_from_page res;
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
    res.status = true;
    res.value.text = fz_string_from_buffer(ctx, buf);
  }
  fz_catch(ctx) {
    res.status = false;
    res.value.err_msg = fz_caught_message(ctx);
  }
  return res;
}

/* Pixmap */
mupdf_pix_as_jpeg mupdf_get_pixmap_as_jpeg(fz_context *ctx, fz_pixmap *pix,
                                           const int quality) {
  mupdf_pix_as_jpeg res;
  unsigned char *data;
  fz_try(ctx) {
    fz_buffer *buf = fz_new_buffer_from_pixmap_as_jpeg(
        ctx, pix, fz_default_color_params, quality, 0);
    const size_t data_len = fz_buffer_storage(ctx, buf, &data);
    res.inner.value.data = data;
    res.inner.value.data_len = data_len;
    res.status = true;
  }
  fz_catch(ctx) {
    res.status = false;
    res.inner.err_msg = fz_caught_message(ctx);
  }
  return res;
}

mupdf_res mupdf_save_pixmap_as_jpeg(fz_context *ctx, fz_pixmap *pix,
                                    const int quality,
                                    const char *path_to_out) {
  mupdf_res res;
  fz_try(ctx) {
    fz_save_pixmap_as_jpeg(ctx, pix, path_to_out, quality);
    res.status = true;
  }
  fz_catch(ctx) {
    res.status = false;
    res.err_msg = fz_caught_message(ctx);
  }
  return res;
}
