import type {
  BackupKeyRequest,
  BackupListRequest,
  BackupSettingsView,
  BackupView,
  CreateBackupRequest,
  ExportResult,
  Page,
  RestoreBackupRequest,
  UpdateBackupSettingsRequest,
} from "./contracts.ts";
import { invokePhase09, requireObject } from "./invokePhase09.ts";

export const backupGateway = {
  async getSettings(): Promise<BackupSettingsView> {
    const result = await invokePhase09<unknown>(
      "phase09_get_backup_settings",
    );
    return requireObject<BackupSettingsView & Record<string, unknown>>(
      result,
      "backup settings",
    );
  },

  async updateSettings(
    request: UpdateBackupSettingsRequest,
  ): Promise<BackupSettingsView> {
    const result = await invokePhase09<unknown>(
      "phase09_update_backup_settings",
      { request },
    );
    return requireObject<BackupSettingsView & Record<string, unknown>>(
      result,
      "updated backup settings",
    );
  },

  async create(request: CreateBackupRequest): Promise<BackupView> {
    const result = await invokePhase09<unknown>("phase09_create_backup", {
      request,
    });
    return requireObject<BackupView & Record<string, unknown>>(
      result,
      "created backup",
    );
  },

  async list(request: BackupListRequest): Promise<Page<BackupView>> {
    const result = await invokePhase09<unknown>("phase09_list_backups", {
      request,
    });
    return requireObject<Page<BackupView> & Record<string, unknown>>(
      result,
      "backup page",
    );
  },

  async verify(request: BackupKeyRequest): Promise<BackupView> {
    const result = await invokePhase09<unknown>("phase09_verify_backup", {
      request,
    });
    return requireObject<BackupView & Record<string, unknown>>(
      result,
      "verified backup",
    );
  },

  async export(request: BackupKeyRequest): Promise<ExportResult> {
    const result = await invokePhase09<unknown>("phase09_export_backup", {
      request,
    });
    return requireObject<ExportResult & Record<string, unknown>>(
      result,
      "backup export",
    );
  },

  async import(): Promise<BackupView> {
    const result = await invokePhase09<unknown>("phase09_import_backup");
    return requireObject<BackupView & Record<string, unknown>>(
      result,
      "imported backup",
    );
  },

  async restore(request: RestoreBackupRequest): Promise<void> {
    if (
      request.confirmationText !== "RESTORE" ||
      request.confirmed !== true ||
      request.currentPassword.length === 0
    ) {
      throw new Error(
        "Restore requires the current password and exact RESTORE confirmation.",
      );
    }
    await invokePhase09<void>("phase09_restore_backup", { request });
  },

  async delete(request: BackupKeyRequest): Promise<void> {
    await invokePhase09<void>("phase09_delete_backup", { request });
  },
};
