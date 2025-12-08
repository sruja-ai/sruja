// apps/website/src/features/search/components/AlgoliaSearch.tsx
import { useState, useEffect } from 'react';
import { liteClient } from 'algoliasearch/lite';
import { envConfig } from '@/config/env';
import { SearchDialog, type SearchItem } from '@sruja/ui';

interface AlgoliaSearchProps {
  isOpen: boolean;
  onClose: () => void;
  onSelect?: (item: SearchItem | null) => void;
}

export default function AlgoliaSearch({ isOpen, onClose, onSelect }: AlgoliaSearchProps) {
  const [searchClient, setSearchClient] = useState<any>(null);

  useEffect(() => {
    if (envConfig.algolia) {
      const client = liteClient(
        envConfig.algolia.appId,
        envConfig.algolia.apiKey
      );
      setSearchClient(client);
      
      // Log configuration in development
      if (envConfig.env === 'development' && typeof console !== 'undefined') {
        console.log(`[Algolia Search] Initialized with index: ${envConfig.algolia.indexName}`);
      }
    } else {
      // Log warning if Algolia is not configured
      if (envConfig.env === 'development' && typeof console !== 'undefined') {
        console.warn('[Algolia Search] Algolia is not configured. Search dialog will open but search will not work.');
      }
    }
  }, []);

  const fetchResults = async (query: string): Promise<SearchItem[]> => {
    if (!searchClient || !envConfig.algolia || !query.trim()) {
      return [];
    }

    try {
      const { results } = await searchClient.search([
        {
          indexName: envConfig.algolia.indexName,
          query,
          params: {
            hitsPerPage: 10,
            attributesToRetrieve: ['title', 'description', 'url', 'type', 'section'],
          },
        },
      ]);

      const hits = results[0]?.hits || [];
      return hits.map((hit: any) => ({
        id: hit.objectID || hit.url || hit.title,
        label: hit.title || 'Untitled',
        subLabel: hit.section || hit.type || hit.description,
        ...(hit.url && { url: hit.url }),
      }));
    } catch (error) {
      console.error('Algolia search error:', error);
      return [];
    }
  };

  const handleSelect = (item: SearchItem | null) => {
    if (item && 'url' in item && (item as any).url) {
      window.location.href = (item as any).url;
    } else if (onSelect) {
      onSelect(item);
    }
  };

  // Always render SearchDialog, but show a message if Algolia is not configured
  return (
    <>
      <SearchDialog
        isOpen={isOpen}
        onClose={onClose}
        fetchResults={envConfig.algolia ? fetchResults : async () => []}
        onSelect={handleSelect}
        verticalOffset={'20vh'}
        badge={envConfig.algolia ? (
          <a
            href="https://www.algolia.com/?utm_medium=AOS-referral"
            target="_blank"
            rel="noopener noreferrer"
            style={{ textDecoration: 'none', color: 'var(--color-primary)', fontWeight: 500 }}
          >
            Search by Algolia
          </a>
        ) : undefined}
      />
    </>
  );
}
