// apps/studio-core/src/context/StudioStateContext.tsx
import React, { createContext, useContext, useState } from 'react';
import type { ReactNode } from 'react';

export interface DocumentationState {
  selectedNodeType: string | null;
  selectedNodeId: string | undefined;
  selectedNodeLabel: string | undefined;
}

export interface SidebarState {
  showSidebar: boolean;
  activePanel: 'explorer' | 'documentation' | 'shortcuts' | 'guide';
  width: number;
}

export interface PropertiesState {
  showProperties: boolean;
}

interface StudioStateContextType {
  // Documentation state
  documentation: DocumentationState;
  setDocumentation: (state: DocumentationState | ((prev: DocumentationState) => DocumentationState)) => void;
  
  // Sidebar state
  sidebar: SidebarState;
  setSidebar: (state: SidebarState | ((prev: SidebarState) => SidebarState)) => void;
  
  // Properties state
  properties: PropertiesState;
  setProperties: (state: PropertiesState | ((prev: PropertiesState) => PropertiesState)) => void;
}

const StudioStateContext = createContext<StudioStateContextType | undefined>(undefined);

export function useStudioState() {
  const context = useContext(StudioStateContext);
  if (!context) {
    throw new Error('useStudioState must be used within StudioStateProvider');
  }
  return context;
}

interface StudioStateProviderProps {
  children: ReactNode;
}

export function StudioStateProvider({ children }: StudioStateProviderProps) {
  // Load initial state from localStorage
  const [documentation, setDocumentation] = useState<DocumentationState>({
    selectedNodeType: null,
    selectedNodeId: undefined,
    selectedNodeLabel: undefined,
  });

  const [sidebar, setSidebar] = useState<SidebarState>(() => {
    const savedVisible = localStorage.getItem('studio-sidebar-visible');
    const savedPanel = localStorage.getItem('studio-sidebar-panel');
    const savedWidth = localStorage.getItem('studio-sidebar-width');
    const allowed = ['explorer', 'documentation', 'shortcuts', 'guide'] as const;
    const initialPanel = allowed.includes((savedPanel as any)) ? (savedPanel as any) : 'explorer';
    return {
      showSidebar: savedVisible !== null ? savedVisible === 'true' : false,
      activePanel: initialPanel,
      width: savedWidth ? parseInt(savedWidth, 10) : 280,
    };
  });

  const [properties, setProperties] = useState<PropertiesState>(() => {
    const saved = localStorage.getItem('studio-properties-visible');
    return {
      showProperties: saved === 'true',
    };
  });

  // Persist sidebar state to localStorage
  React.useEffect(() => {
    localStorage.setItem('studio-sidebar-visible', String(sidebar.showSidebar));
  }, [sidebar.showSidebar]);

  React.useEffect(() => {
    localStorage.setItem('studio-sidebar-panel', sidebar.activePanel);
  }, [sidebar.activePanel]);

  React.useEffect(() => {
    localStorage.setItem('studio-sidebar-width', String(sidebar.width));
  }, [sidebar.width]);

  React.useEffect(() => {
    localStorage.setItem('studio-properties-visible', String(properties.showProperties));
  }, [properties.showProperties]);

  const value: StudioStateContextType = {
    documentation,
    setDocumentation,
    sidebar,
    setSidebar,
    properties,
    setProperties,
  };

  return (
    <StudioStateContext.Provider value={value}>
      {children}
    </StudioStateContext.Provider>
  );
}






