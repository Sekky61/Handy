import React, { useMemo } from "react";
import { useTranslation } from "react-i18next";
import { RefreshCcw } from "lucide-react";

import { Dropdown, SettingContainer, SettingsGroup } from "@/components/ui";
import { ResetButton } from "@/components/ui/ResetButton";
import { useSettings } from "@/hooks/useSettings";
import { ProviderSelect } from "../PostProcessingSettingsApi/ProviderSelect";
import { ApiKeyField } from "../PostProcessingSettingsApi/ApiKeyField";
import { ModelSelect } from "../PostProcessingSettingsApi/ModelSelect";
import type { SttBackend } from "@/bindings";
import type { ModelOption } from "../PostProcessingSettingsApi/types";
import type { DropdownOption } from "@/components/ui/Dropdown";

export const SttSettings: React.FC = () => {
  const { t } = useTranslation();
  const {
    settings,
    updateSetting,
    isUpdating,
    setSttProvider,
    updateSttApiKey,
    updateSttModel,
    fetchSttModels,
    sttModelOptions,
  } = useSettings();

  const providers = settings?.stt_providers ?? [];
  const selectedProviderId =
    settings?.stt_provider_id ?? providers[0]?.id ?? "openrouter";
  const selectedProvider =
    providers.find((provider) => provider.id === selectedProviderId) ??
    providers[0];
  const apiKey = settings?.stt_api_keys?.[selectedProviderId] ?? "";
  const model = settings?.stt_models?.[selectedProviderId] ?? "";
  const backend = settings?.stt_backend ?? "local";

  const providerOptions = useMemo<DropdownOption[]>(() => {
    return providers.map((provider) => ({
      value: provider.id,
      label: provider.label,
    }));
  }, [providers]);

  const backendOptions = useMemo<DropdownOption[]>(
    () => [
      { value: "local", label: t("settings.stt.backend.local") },
      { value: "open_router", label: t("settings.stt.backend.openRouter") },
    ],
    [t],
  );

  const modelOptions = useMemo<ModelOption[]>(() => {
    const seen = new Set<string>();
    const options: ModelOption[] = [];
    const add = (value: string | null | undefined) => {
      const trimmed = value?.trim();
      if (!trimmed || seen.has(trimmed)) return;
      seen.add(trimmed);
      options.push({ value: trimmed, label: trimmed });
    };

    for (const candidate of sttModelOptions[selectedProviderId] ?? []) {
      add(candidate);
    }
    add(model);

    return options;
  }, [model, selectedProviderId, sttModelOptions]);

  const isOpenRouter = backend === "open_router";
  const isApiKeyUpdating = isUpdating(`stt_api_key:${selectedProviderId}`);
  const isModelUpdating = isUpdating(`stt_model:${selectedProviderId}`);
  const isFetchingModels = isUpdating(`stt_models_fetch:${selectedProviderId}`);

  const handleBackendSelect = (value: string) => {
    void updateSetting("stt_backend", value as SttBackend);
  };

  const handleProviderSelect = (providerId: string) => {
    if (providerId === selectedProviderId) return;
    void setSttProvider(providerId);
  };

  const handleApiKeyChange = (value: string) => {
    const trimmed = value.trim();
    if (trimmed !== apiKey) {
      void updateSttApiKey(selectedProviderId, trimmed);
    }
  };

  const handleModelSelect = (value: string) => {
    void updateSttModel(selectedProviderId, value.trim());
  };

  const handleRefreshModels = () => {
    void fetchSttModels(selectedProviderId);
  };

  return (
    <div className="max-w-3xl w-full mx-auto space-y-6">
      <SettingsGroup title={t("settings.stt.title")}>
        <SettingContainer
          title={t("settings.stt.backend.title")}
          description={t("settings.stt.backend.description")}
          descriptionMode="tooltip"
          layout="horizontal"
          grouped={true}
        >
          <Dropdown
            options={backendOptions}
            selectedValue={backend}
            onSelect={handleBackendSelect}
            disabled={isUpdating("stt_backend")}
            className="flex-1"
          />
        </SettingContainer>
      </SettingsGroup>

      {isOpenRouter && (
        <SettingsGroup title={t("settings.stt.api.title")}>
          <SettingContainer
            title={t("settings.stt.api.provider.title")}
            description={t("settings.stt.api.provider.description")}
            descriptionMode="tooltip"
            layout="horizontal"
            grouped={true}
          >
            <ProviderSelect
              options={providerOptions}
              value={selectedProviderId}
              onChange={handleProviderSelect}
              disabled={
                providerOptions.length <= 1 || isUpdating("stt_provider_id")
              }
            />
          </SettingContainer>

          <SettingContainer
            title={t("settings.stt.api.apiKey.title")}
            description={t("settings.stt.api.apiKey.description")}
            descriptionMode="tooltip"
            layout="horizontal"
            grouped={true}
          >
            <ApiKeyField
              value={apiKey}
              onBlur={handleApiKeyChange}
              placeholder={t("settings.stt.api.apiKey.placeholder")}
              disabled={isApiKeyUpdating}
              className="min-w-[320px]"
            />
          </SettingContainer>

          <SettingContainer
            title={t("settings.stt.api.model.title")}
            description={t("settings.stt.api.model.description")}
            descriptionMode="tooltip"
            layout="stacked"
            grouped={true}
          >
            <div className="flex items-center gap-2">
              <ModelSelect
                value={model}
                options={modelOptions}
                disabled={isModelUpdating || !selectedProvider}
                isLoading={isFetchingModels}
                placeholder={
                  modelOptions.length > 0
                    ? t("settings.stt.api.model.placeholderWithOptions")
                    : t("settings.stt.api.model.placeholderNoOptions")
                }
                onSelect={handleModelSelect}
                onCreate={handleModelSelect}
                onBlur={() => {}}
                className="flex-1 min-w-[380px]"
              />
              <ResetButton
                onClick={handleRefreshModels}
                disabled={isFetchingModels || !selectedProvider}
                ariaLabel={t("settings.stt.api.model.refreshModels")}
                className="flex h-10 w-10 items-center justify-center"
              >
                <RefreshCcw
                  className={`h-4 w-4 ${isFetchingModels ? "animate-spin" : ""}`}
                />
              </ResetButton>
            </div>
          </SettingContainer>
        </SettingsGroup>
      )}
    </div>
  );
};
