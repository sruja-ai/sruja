// packages/ui/src/components/index.ts
// Core components (not provided by Headless UI)
export { Logo } from './Logo';
export type { LogoProps } from './Logo';
export { Button } from './Button';
export type { ButtonProps } from './Button';
export { Header } from './Header';
export type { HeaderProps } from './Header';
export { Footer } from './Footer';
export type { FooterProps } from './Footer';

// Theme system
export { ThemeProvider, useTheme } from './ThemeProvider';
export type { ThemeProviderProps } from './ThemeProvider';
export { ThemeToggle } from './ThemeToggle';
export type { ThemeToggleProps } from './ThemeToggle';

// Styled Headless UI wrappers (optional convenience)
export { Dialog } from './Dialog';
export type { DialogProps } from './Dialog';
export { Menu } from './Menu';
export type { MenuProps, MenuItem as MenuItemType } from './Menu';
export { Popover } from './Popover';
export type { PopoverProps } from './Popover';

// Headless UI components (unstyled, for advanced usage)
export * from './headless-ui';

export { Card } from './Card';
export type { CardProps } from './Card';
export { Badge } from './Badge';
export type { BadgeProps } from './Badge';
export { Input } from './Input';
export type { InputProps } from './Input';
export { AppShell } from './AppShell';
export type { AppShellProps } from './AppShell';
export { Tabs } from './Tabs';
export type { TabsProps } from './Tabs';
export { Combobox } from './Combobox';
export type { ComboboxProps, ComboOption } from './Combobox';
export { Disclosure } from './Disclosure';
export type { DisclosureProps } from './Disclosure';
export { Switch } from './Switch';
export type { SwitchProps } from './Switch';
export { RadioGroup } from './RadioGroup';
export type { RadioGroupProps, RadioOption } from './RadioGroup';
export { Listbox } from './Listbox';
export type { ListboxProps, ListOption } from './Listbox';
export { Skeleton } from './Skeleton';
export type { SkeletonProps } from './Skeleton';
export { SearchBar } from './SearchBar';
export type { SearchBarProps, SearchItem } from './SearchBar';
export { SearchDialog } from './SearchDialog';
export type { SearchDialogProps } from './SearchDialog';
export { Breadcrumb } from './Breadcrumb';
export type { BreadcrumbProps, BreadcrumbItem } from './Breadcrumb';

export { MonacoEditor } from './MonacoEditor';
export type { MonacoEditorProps } from './MonacoEditor';
export { SrujaMonacoEditor } from './SrujaMonacoEditor';
export type { SrujaMonacoEditorProps } from './SrujaMonacoEditor';
export { PosthogProvider, usePosthog } from './PosthogProvider';
export { MermaidDiagram } from './MermaidDiagram';
export type { MermaidDiagramProps } from './MermaidDiagram';
export { MarkdownPreview } from './MarkdownPreview';
export type { MarkdownPreviewProps } from './MarkdownPreview';
export { SrujaLoader } from './SrujaLoader';
export type { SrujaLoaderProps } from './SrujaLoader';
