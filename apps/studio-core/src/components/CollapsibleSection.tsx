// apps/studio-core/src/components/CollapsibleSection.tsx
import React, { useState } from 'react';
import { ChevronDown, ChevronUp } from 'lucide-react';

interface CollapsibleSectionProps {
  title: string;
  icon?: React.ReactNode;
  defaultExpanded?: boolean;
  children: React.ReactNode;
  headerActions?: React.ReactNode;
}

export const CollapsibleSection: React.FC<CollapsibleSectionProps> = ({
  title,
  icon,
  defaultExpanded = true,
  children,
  headerActions,
}) => {
  const [isExpanded, setIsExpanded] = useState(defaultExpanded);

  return (
    <div className="flex flex-col border-b border-gray-800 last:border-b-0">
      <button
        onClick={() => setIsExpanded(!isExpanded)}
        className="flex items-center justify-between px-4 py-3 text-xs font-semibold text-gray-400 uppercase tracking-wider hover:bg-gray-800/50 transition-all duration-200 group"
      >
        <div className="flex items-center gap-2">
          {icon && <div className="text-gray-500 group-hover:text-gray-400 transition-colors">{icon}</div>}
          <span className="group-hover:text-gray-300 transition-colors">{title}</span>
        </div>
        <div className="flex items-center gap-2">
          {headerActions}
          {isExpanded ? (
            <ChevronUp size={14} className="text-gray-500 group-hover:text-gray-400 transition-all duration-200" />
          ) : (
            <ChevronDown size={14} className="text-gray-500 group-hover:text-gray-400 transition-all duration-200" />
          )}
        </div>
      </button>
      {isExpanded && (
        <div className="flex-shrink-0">
          {children}
        </div>
      )}
    </div>
  );
};



