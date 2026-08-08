import type {
  ExportResult,
  ReportDescriptor,
  ReportPage,
  ReportRequest,
} from "./contracts.ts";
import {
  invokePhase09,
  requireArray,
  requireObject,
} from "./invokePhase09.ts";

export const reportsGateway = {
  async list(): Promise<ReportDescriptor[]> {
    const result = await invokePhase09<unknown>("phase09_list_reports");
    return requireArray<ReportDescriptor>(result, "report list");
  },

  async run(request: ReportRequest): Promise<ReportPage> {
    const result = await invokePhase09<unknown>("phase09_run_report", {
      request,
    });
    return requireObject<ReportPage & Record<string, unknown>>(
      result,
      "report page",
    );
  },

  async exportCsv(request: ReportRequest): Promise<ExportResult> {
    const result = await invokePhase09<unknown>(
      "phase09_export_report_csv",
      { request },
    );
    return requireObject<ExportResult & Record<string, unknown>>(
      result,
      "report CSV export",
    );
  },

  async exportPdf(request: ReportRequest): Promise<ExportResult> {
    const result = await invokePhase09<unknown>(
      "phase09_export_report_pdf",
      { request },
    );
    return requireObject<ExportResult & Record<string, unknown>>(
      result,
      "report PDF export",
    );
  },
};
