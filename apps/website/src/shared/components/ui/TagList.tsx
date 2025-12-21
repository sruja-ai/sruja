// apps/website/src/shared/components/ui/TagList.tsx
interface TagListProps {
  tags: string[];
  className?: string;
}

export function TagList({ tags, className = '' }: TagListProps) {
  if (!tags || tags.length === 0) return null;

  return (
    <div className={`tags ${className}`}>
      {tags.map((tag: string) => (
        <span key={tag} className="tag">{tag}</span>
      ))}
    </div>
  );
}
