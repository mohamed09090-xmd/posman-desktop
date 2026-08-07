import type {
  DocumentRequest,
  ExportResult,
  Page,
  PreviewContent,
  PreviewResult,
  RenderedDocumentKeyRequest,
  RenderedDocumentsRequest,
  RenderedDocumentView,
} from "./contracts";
import { invokePhase09, requireObject } from "./invokePhase09";

export const documentsGateway = {
  async preview(request: DocumentRequest): Promise<PreviewResult> {
    const result = await invokePhase09<unknown>("phase09_preview_document", {
      request,
    });
    return requireObject<PreviewResult & Record<string, unknown>>(
      result,
      "document preview",
    );
  },

  async getPreviewContent(previewId: string): Promise<PreviewContent> {
    const result = await invokePhase09<unknown>(
      "phase09_get_preview_content",
      { request: previewId },
    );
    return requireObject<PreviewContent & Record<string, unknown>>(
      result,
      "preview content",
    );
  },

  async render(request: DocumentRequest): Promise<RenderedDocumentView> {
    const result = await invokePhase09<unknown>("phase09_render_document", {
      request,
    });
    return requireObject<RenderedDocumentView & Record<string, unknown>>(
      result,
      "rendered document",
    );
  },

  async list(
    request: RenderedDocumentsRequest,
  ): Promise<Page<RenderedDocumentView>> {
    const result = await invokePhase09<unknown>(
      "phase09_list_rendered_documents",
      { request },
    );
    return requireObject<Page<RenderedDocumentView> & Record<string, unknown>>(
      result,
      "rendered document page",
    );
  },

  async get(
    request: RenderedDocumentKeyRequest,
  ): Promise<RenderedDocumentView> {
    const result = await invokePhase09<unknown>(
      "phase09_get_rendered_document",
      { request },
    );
    return requireObject<RenderedDocumentView & Record<string, unknown>>(
      result,
      "rendered document",
    );
  },

  async verify(
    request: RenderedDocumentKeyRequest,
  ): Promise<RenderedDocumentView> {
    const result = await invokePhase09<unknown>(
      "phase09_verify_rendered_document",
      { request },
    );
    return requireObject<RenderedDocumentView & Record<string, unknown>>(
      result,
      "verified rendered document",
    );
  },

  async exportPdf(request: RenderedDocumentKeyRequest): Promise<ExportResult> {
    const result = await invokePhase09<unknown>(
      "phase09_export_rendered_pdf",
      { request },
    );
    return requireObject<ExportResult & Record<string, unknown>>(
      result,
      "document export",
    );
  },

  async print(request: RenderedDocumentKeyRequest): Promise<void> {
    await invokePhase09<void>("phase09_print_rendered_document", { request });
  },
};
