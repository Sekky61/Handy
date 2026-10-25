import React, { useState } from "react";
import { BookmarkPlus, Check, RotateCcw, Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { commands, type SttPreset } from "@/bindings";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { SettingsGroup } from "@/components/ui/SettingsGroup";
import { useSettings } from "@/hooks/useSettings";
import { notify } from "@/stores/noticeStore";

export const SttPresets: React.FC = () => {
  const { t } = useTranslation();
  const { settings, refreshSettings } = useSettings();
  const [name, setName] = useState("");
  const [busyKey, setBusyKey] = useState<string | null>(null);
  const presets = settings?.stt_presets ?? [];

  const run = async (
    key: string,
    operation: () => Promise<{ status: string; error?: string }>,
  ) => {
    setBusyKey(key);
    try {
      const result = await operation();
      if (result.status === "error") {
        notify({
          title: t("settings.stt.presets.error"),
          detail: result.error,
        });
        return false;
      }
      await refreshSettings();
      return true;
    } catch (error) {
      notify({
        title: t("settings.stt.presets.error"),
        detail: String(error),
      });
      return false;
    } finally {
      setBusyKey(null);
    }
  };

  const handleSave = async () => {
    if (!name.trim()) return;
    const saved = await run("save", () => commands.saveSttPreset(name.trim()));
    if (saved) setName("");
  };

  const handleActivate = (preset: SttPreset) =>
    run(`activate:${preset.id}`, () => commands.activateSttPreset(preset.id));

  const handleUpdate = (preset: SttPreset) =>
    run(`update:${preset.id}`, () =>
      commands.updateSttPreset(preset.id, preset.name),
    );

  const handleDelete = (preset: SttPreset) =>
    run(`delete:${preset.id}`, () => commands.deleteSttPreset(preset.id));

  const describePreset = (preset: SttPreset) => {
    const promptName = settings?.post_process_prompts?.find(
      (prompt) => prompt.id === preset.post_process_prompt_id,
    )?.name;
    const sttModel =
      preset.backend === "local"
        ? preset.model
        : `${preset.stt_provider_id}: ${preset.model}`;
    const postProcess = preset.post_process_enabled
      ? `${preset.post_process_provider_id}: ${preset.post_process_model}${promptName ? ` · ${promptName}` : ""}`
      : t("settings.stt.presets.postProcessingOff");

    return `${sttModel} · ${postProcess}`;
  };

  return (
    <SettingsGroup
      title={t("settings.stt.presets.title")}
      description={t("settings.stt.presets.description")}
    >
      <div className="px-4 py-3 space-y-3">
        <div className="flex gap-2">
          <Input
            value={name}
            onChange={(event) => setName(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "Enter") void handleSave();
            }}
            placeholder={t("settings.stt.presets.namePlaceholder")}
            aria-label={t("settings.stt.presets.namePlaceholder")}
            maxLength={60}
            className="flex-1 min-w-0"
          />
          <Button
            onClick={() => void handleSave()}
            disabled={!name.trim() || busyKey !== null}
            className="flex items-center gap-2 shrink-0"
          >
            <BookmarkPlus className="w-4 h-4" />
            {t("settings.stt.presets.saveCurrent")}
          </Button>
        </div>

        {presets.length === 0 ? (
          <p className="text-sm text-mid-gray">
            {t("settings.stt.presets.empty")}
          </p>
        ) : (
          <div className="space-y-2">
            {presets.map((preset) => (
              <div
                key={preset.id}
                className="flex flex-col sm:flex-row sm:items-center gap-3 rounded-lg border border-mid-gray/20 px-3 py-2"
              >
                <div className="min-w-0 flex-1">
                  <div className="font-medium text-sm truncate">
                    {preset.name}
                  </div>
                  <div className="text-xs text-mid-gray truncate">
                    {describePreset(preset)}
                  </div>
                </div>
                <div className="flex flex-wrap gap-2 shrink-0">
                  <Button
                    variant="primary-soft"
                    size="sm"
                    onClick={() => void handleActivate(preset)}
                    disabled={busyKey !== null}
                    className="flex items-center gap-1"
                  >
                    <Check className="w-3.5 h-3.5" />
                    {t("settings.stt.presets.use")}
                  </Button>
                  <Button
                    variant="secondary"
                    size="sm"
                    onClick={() => void handleUpdate(preset)}
                    disabled={busyKey !== null}
                    className="flex items-center gap-1"
                  >
                    <RotateCcw className="w-3.5 h-3.5" />
                    {t("settings.stt.presets.update")}
                  </Button>
                  <Button
                    variant="danger-ghost"
                    size="sm"
                    onClick={() => void handleDelete(preset)}
                    disabled={busyKey !== null}
                    aria-label={t("settings.stt.presets.delete", {
                      name: preset.name,
                    })}
                    title={t("settings.stt.presets.delete", {
                      name: preset.name,
                    })}
                  >
                    <Trash2 className="w-4 h-4" />
                  </Button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </SettingsGroup>
  );
};
