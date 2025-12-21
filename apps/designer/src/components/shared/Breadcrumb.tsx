import { Breadcrumb as UIBreadcrumb, type BreadcrumbItem } from "@sruja/ui";
import { useViewStore } from "../../stores";

export function Breadcrumb() {
  const breadcrumb = useViewStore((s) => s.breadcrumb);
  const goToRoot = useViewStore((s) => s.goToRoot);
  const goUp = useViewStore((s) => s.goUp);

  if (!breadcrumb || !Array.isArray(breadcrumb)) {
    return null;
  }

  const items: BreadcrumbItem[] = breadcrumb.map((label, index) => ({ id: String(index), label }));

  return (
    <UIBreadcrumb
      items={items}
      onItemClick={(id) => {
        const targetIndex = Number(id);
        if (Number.isNaN(targetIndex)) return;
        const stepsBack = breadcrumb.length - 1 - targetIndex;
        for (let i = 0; i < stepsBack; i++) {
          goUp();
        }
      }}
      onHomeClick={goToRoot}
    />
  );
}
