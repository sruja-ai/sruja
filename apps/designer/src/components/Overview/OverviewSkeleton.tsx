// apps/designer/src/components/Overview/OverviewSkeleton.tsx
import { Skeleton, SkeletonCard } from "../shared/Skeleton";
import "./OverviewSkeleton.css";

export function OverviewSkeleton() {
  return (
    <div className="overview-skeleton">
      {/* Hero Section */}
      <div className="overview-skeleton-hero">
        <Skeleton variant="title" width="60%" />
        <Skeleton variant="text" />
        <Skeleton variant="text" width="80%" />
      </div>

      {/* Governance Widget */}
      <div className="overview-skeleton-card">
        <Skeleton variant="title" width="30%" />
        <div className="overview-skeleton-stats">
          <SkeletonCard lines={2} />
        </div>
      </div>

      {/* Stats Row */}
      <div className="overview-skeleton-stats-row">
        {[1, 2, 3, 4].map((i) => (
          <div key={i} className="overview-skeleton-stat">
            <Skeleton variant="badge" />
            <Skeleton variant="text" />
          </div>
        ))}
      </div>

      {/* Quick Navigation Cards */}
      <div className="overview-skeleton-cards">
        {[1, 2, 3, 4, 5].map((i) => (
          <div key={i} className="overview-skeleton-nav-card">
            <Skeleton variant="title" />
            <Skeleton variant="text" />
            <Skeleton variant="button" />
          </div>
        ))}
      </div>
    </div>
  );
}
