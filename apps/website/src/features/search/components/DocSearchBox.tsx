// apps/website/src/features/search/components/DocSearchBox.tsx
import { DocSearch } from "@docsearch/react";
import "@docsearch/css";
import { envConfig } from "@/config/env";
import "./DocSearchBox.css";

export default function DocSearchBox() {
  if (!envConfig.algolia) return null;
  const { appId, apiKey, indexName } = envConfig.algolia;
  return (
    <div className="docsearch-wrapper">
      <DocSearch
        appId={appId}
        apiKey={apiKey}
        // indexName is deprecated in @docsearch/react but still works in v4.3.2
        // Using type assertion to suppress deprecation warning until we upgrade
        {...({ indexName } as Record<string, unknown>)}
        translations={{
          button: {
            buttonText: "Search",
            buttonAriaLabel: "Search documentation",
          },
        }}
      />
    </div>
  );
}
