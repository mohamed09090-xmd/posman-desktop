import type {
  CreateTemplateDraftRequest,
  PublishTemplateRequest,
  RetireTemplateRequest,
  TemplateDetail,
  TemplateDraftView,
  TemplateKeyRequest,
  TemplateSummary,
  TemplateVersionView,
  UpdateTemplateDraftRequest,
} from "./contracts.ts";
import {
  invokePhase09,
  requireArray,
  requireObject,
} from "./invokePhase09.ts";

export const templatesGateway = {
  async list(): Promise<TemplateSummary[]> {
    const result = await invokePhase09<unknown>("phase09_list_templates");
    return requireArray<TemplateSummary>(result, "template list");
  },

  async get(request: TemplateKeyRequest): Promise<TemplateDetail> {
    const result = await invokePhase09<unknown>("phase09_get_template", {
      request,
    });
    return requireObject<TemplateDetail & Record<string, unknown>>(
      result,
      "template detail",
    );
  },

  async createDraft(
    request: CreateTemplateDraftRequest,
  ): Promise<TemplateDraftView> {
    const result = await invokePhase09<unknown>(
      "phase09_create_template_draft",
      { request },
    );
    return requireObject<TemplateDraftView & Record<string, unknown>>(
      result,
      "template draft",
    );
  },

  async updateDraft(
    request: UpdateTemplateDraftRequest,
  ): Promise<TemplateDraftView> {
    const result = await invokePhase09<unknown>(
      "phase09_update_template_draft",
      { request },
    );
    return requireObject<TemplateDraftView & Record<string, unknown>>(
      result,
      "updated template draft",
    );
  },

  async publish(
    request: PublishTemplateRequest,
  ): Promise<TemplateVersionView> {
    const result = await invokePhase09<unknown>("phase09_publish_template", {
      request,
    });
    return requireObject<TemplateVersionView & Record<string, unknown>>(
      result,
      "published template version",
    );
  },

  async retire(
    request: RetireTemplateRequest,
  ): Promise<TemplateVersionView> {
    const result = await invokePhase09<unknown>("phase09_retire_template", {
      request,
    });
    return requireObject<TemplateVersionView & Record<string, unknown>>(
      result,
      "retired template version",
    );
  },
};
