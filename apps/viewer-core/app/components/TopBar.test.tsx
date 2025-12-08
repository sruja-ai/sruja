import { render, screen, fireEvent } from '@testing-library/react'
import { describe, it, expect, vi } from 'vitest'
import { TopBar } from './TopBar'

vi.mock('@sruja/ui', () => ({
  ThemeToggle: () => <div data-testid="theme-toggle" />, 
  Logo: () => <div data-testid="logo" />, 
  Button: (props: any) => <button {...props} />,
  Breadcrumb: ({ items, onItemClick, onHomeClick }: any) => (
    <div>
      <button aria-label="home" onClick={() => onHomeClick && onHomeClick()}>Home</button>
      {items?.map((it: any) => (
        <button key={it.id} onClick={() => onItemClick && onItemClick(it.id)}>{it.label}</button>
      ))}
    </div>
  ),
  SearchBar: ({ onSelect }: any) => (
    <button aria-label="search-select" onClick={() => onSelect && onSelect({ id: 'WebApp', label: 'Web App' })}>Select</button>
  ),
}))

const data = {
  metadata: { name: 'Test Arch' },
  architecture: {
    systems: [{ id: 'WebApp', label: 'Web App', containers: [{ id: 'API', label: 'API' }] }],
    persons: [{ id: 'User', label: 'User' }],
    requirements: [{ id: 'REQ-1', title: 'Login' }],
    adrs: [{ id: 'ADR-1', title: 'Use Postgres' }],
  },
} as any

describe('TopBar', () => {
  it('calls onSetLevel when clicking level buttons', () => {
    const onSetLevel = vi.fn()
    render(
      <TopBar
        data={data}
        currentLevel={1}
        onSetLevel={onSetLevel}
        onSearch={() => {}}
        onSelectNode={() => {}}
        breadcrumbs={[]}
        onBreadcrumbClick={() => {}}
      />
    )

    fireEvent.click(screen.getByText('Container'))
    fireEvent.click(screen.getByText('Component'))
    expect(onSetLevel).toHaveBeenCalledWith(2)
    expect(onSetLevel).toHaveBeenCalledWith(3)
  })

  it('calls onLayoutChange on layout select change', () => {
    const onLayoutChange = vi.fn()
    render(
      <TopBar
        data={data}
        currentLevel={1}
        onSetLevel={() => {}}
        onSearch={() => {}}
        onSelectNode={() => {}}
        breadcrumbs={[]}
        onBreadcrumbClick={() => {}}
        onLayoutChange={onLayoutChange}
      />
    )

    const select = screen.getByRole('combobox') as HTMLSelectElement
    fireEvent.change(select, { target: { value: 'elk' } })
    expect(onLayoutChange).toHaveBeenCalledWith('elk')
  })

  it('triggers breadcrumb click and home click', () => {
    const onBreadcrumbClick = vi.fn()
    render(
      <TopBar
        data={data}
        currentLevel={1}
        onSetLevel={() => {}}
        onSearch={() => {}}
        onSelectNode={() => {}}
        breadcrumbs={[{ id: 'WebApp', label: 'Web App' }]}
        onBreadcrumbClick={onBreadcrumbClick}
      />
    )

    fireEvent.click(screen.getByText('Web App'))
    expect(onBreadcrumbClick).toHaveBeenCalledWith('WebApp')

    fireEvent.click(screen.getByLabelText('home'))
    expect(onBreadcrumbClick).toHaveBeenCalledWith('root')
  })
})
