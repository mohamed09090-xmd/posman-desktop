import type {
  AuditEventView,
  AuditRequest,
  ExportResult,
  Page,
} from "./contracts.ts";
import { invokePhase09, requireObject } from "./invokePhase09.ts";

export const auditGateway = {
  async list(request: AuditRequest): Promise<Page<AuditEventView>> {
    const result = await invokePhase09<unknown>("phase09_list_audit_events", {
      request,
    });
    return requireObject<Page<AuditEventView> & Record<string, unknown>>(
      result,
      "audit page",
    );
  },

  async exportCsv(request: AuditRequest): Promise<ExportResult> {
    const result = await invokePhase09<unknown>("phase09_export_audit_csv", {
      request,
    });
    return requireObject<ExportResult & Record<string, unknown>>(
      result,
      "audit CSV export",
    );
  },
};
