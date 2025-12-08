import { Skeleton } from '../../../../packages/ui/src/components/Skeleton'

export default { title: 'Feedback/Skeleton', component: Skeleton, tags: ['autodocs'] }

export const Blocks = () => (
  <div style={{ display: 'grid', gap: 12 }}>
    <Skeleton className="h-4 w-48" />
    <Skeleton className="h-4 w-64" />
    <Skeleton className="h-24 w-full" />
  </div>
)

