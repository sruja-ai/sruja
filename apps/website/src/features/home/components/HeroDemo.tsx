/**
 * Client-only island: live editor + diagram for the homepage.
 * Renders nothing until mounted so the static hero content is always visible first.
 */
import LiveSrujaBlock from "@/features/playground/components/LiveSrujaBlock";
import { HERO_INITIAL_DSL } from "../heroInitialDsl";

export default function HeroDemo() {
  return <LiveSrujaBlock initialDsl={HERO_INITIAL_DSL} />;
}
