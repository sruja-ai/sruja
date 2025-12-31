// apps/designer/src/stores/__tests__/viewStore.test.ts
import { describe, it, expect, beforeEach } from "vitest";
import { useViewStore, useSelectionStore } from "../viewStore";
import type { C4Level } from "../../types";
import type { FlowDump } from "@sruja/shared";

describe("viewStore", () => {
  beforeEach(() => {
    // Reset view store
    useViewStore.setState({
      currentLevel: "L1",
      focusedSystemId: null,
      focusedContainerId: null,
      expandedNodes: new Set<string>(),
      breadcrumb: ["Architecture"],
      activeViewId: null,
    });

    // Reset selection store
    useSelectionStore.setState({
      selectedNodeId: null,
      activeFlow: null,
      activeRequirement: null,
      flowStep: 0,
      isFlowPlaying: false,
    });
  });

  describe("useViewStore", () => {
    it("should initialize with default values", () => {
      const state = useViewStore.getState();

      expect(state.currentLevel).toBe("L1");
      expect(state.focusedSystemId).toBeNull();
      expect(state.focusedContainerId).toBeNull();
      expect(state.expandedNodes.size).toBe(0);
      expect(state.breadcrumb).toEqual(["Architecture"]);
      expect(state.activeViewId).toBeNull();
    });

    it("should set level", () => {
      const levels: C4Level[] = ["L1", "L2", "L3", "L4"];

      levels.forEach((level) => {
        useViewStore.getState().setLevel(level);
        expect(useViewStore.getState().currentLevel).toBe(level);
      });
    });

    it("should clear focused IDs when setting level to L1", () => {
      useViewStore.setState({
        focusedSystemId: "System1",
        focusedContainerId: "Container1",
      });

      useViewStore.getState().setLevel("L1");

      const state = useViewStore.getState();
      expect(state.focusedSystemId).toBeNull();
      expect(state.focusedContainerId).toBeNull();
      expect(state.breadcrumb).toEqual(["Architecture"]);
    });

    it("should preserve focusedSystemId when setting level to L2", () => {
      useViewStore.setState({
        focusedSystemId: "System1",
        focusedContainerId: "Container1",
      });

      useViewStore.getState().setLevel("L2");

      const state = useViewStore.getState();
      expect(state.focusedSystemId).toBe("System1");
      expect(state.focusedContainerId).toBeNull();
      expect(state.breadcrumb).toEqual(["Architecture", "System1"]);
    });

    it("should preserve both IDs when setting level to L3", () => {
      useViewStore.setState({
        focusedSystemId: "System1",
        focusedContainerId: "Container1",
      });

      useViewStore.getState().setLevel("L3");

      const state = useViewStore.getState();
      expect(state.focusedSystemId).toBe("System1");
      expect(state.focusedContainerId).toBe("Container1");
      expect(state.breadcrumb).toEqual(["Architecture", "System1", "Container1"]);
    });

    it("should drill down to system", () => {
      useViewStore.getState().drillDown("System1", "system");

      const state = useViewStore.getState();
      expect(state.currentLevel).toBe("L2");
      expect(state.focusedSystemId).toBe("System1");
      expect(state.focusedContainerId).toBeNull();
      expect(state.breadcrumb).toEqual(["Architecture", "System1"]);
    });

    it("should drill down to container", () => {
      useViewStore.setState({
        focusedSystemId: "System1",
      });

      useViewStore.getState().drillDown("Container1", "container", "System1");

      const state = useViewStore.getState();
      expect(state.currentLevel).toBe("L3");
      expect(state.focusedSystemId).toBe("System1");
      expect(state.focusedContainerId).toBe("Container1");
      expect(state.breadcrumb).toEqual(["Architecture", "System1", "Container1"]);
    });

    it("should go up from L3 to L2", () => {
      useViewStore.setState({
        currentLevel: "L3",
        focusedSystemId: "System1",
        focusedContainerId: "Container1",
        breadcrumb: ["Architecture", "System1", "Container1"],
      });

      useViewStore.getState().goUp();

      const state = useViewStore.getState();
      expect(state.currentLevel).toBe("L2");
      expect(state.focusedSystemId).toBe("System1");
      expect(state.focusedContainerId).toBeNull();
      expect(state.breadcrumb).toEqual(["Architecture", "System1"]);
    });

    it("should go up from L2 to L1", () => {
      useViewStore.setState({
        currentLevel: "L2",
        focusedSystemId: "System1",
        breadcrumb: ["Architecture", "System1"],
      });

      useViewStore.getState().goUp();

      const state = useViewStore.getState();
      expect(state.currentLevel).toBe("L1");
      expect(state.focusedSystemId).toBeNull();
      expect(state.breadcrumb).toEqual(["Architecture"]);
    });

    it("should go to root", () => {
      useViewStore.setState({
        currentLevel: "L3",
        focusedSystemId: "System1",
        focusedContainerId: "Container1",
        breadcrumb: ["Architecture", "System1", "Container1"],
      });

      useViewStore.getState().goToRoot();

      const state = useViewStore.getState();
      expect(state.currentLevel).toBe("L1");
      expect(state.focusedSystemId).toBeNull();
      expect(state.focusedContainerId).toBeNull();
      expect(state.breadcrumb).toEqual(["Architecture"]);
    });

    it("should toggle expand node", () => {
      useViewStore.getState().toggleExpand("Node1");
      expect(useViewStore.getState().expandedNodes.has("Node1")).toBe(true);

      useViewStore.getState().toggleExpand("Node1");
      expect(useViewStore.getState().expandedNodes.has("Node1")).toBe(false);
    });

    it("should set active view", () => {
      useViewStore.getState().setActiveView("view1");
      expect(useViewStore.getState().activeViewId).toBe("view1");

      useViewStore.getState().setActiveView(null);
      expect(useViewStore.getState().activeViewId).toBeNull();
    });

    it("should clear active view when setting level", () => {
      useViewStore.getState().setActiveView("view1");
      useViewStore.getState().setLevel("L2");
      expect(useViewStore.getState().activeViewId).toBeNull();
    });
  });

  describe("useSelectionStore", () => {
    it("should initialize with default values", () => {
      const state = useSelectionStore.getState();

      expect(state.selectedNodeId).toBeNull();
      expect(state.activeFlow).toBeNull();
      expect(state.activeRequirement).toBeNull();
      expect(state.flowStep).toBe(0);
      expect(state.isFlowPlaying).toBe(false);
    });

    it("should select node", () => {
      useSelectionStore.getState().selectNode("Node1");
      expect(useSelectionStore.getState().selectedNodeId).toBe("Node1");

      useSelectionStore.getState().selectNode(null);
      expect(useSelectionStore.getState().selectedNodeId).toBeNull();
    });

    it("should set active flow", () => {
      const flow: FlowDump = {
        id: "flow1",
        title: "Test Flow",
        steps: [],
      };

      useSelectionStore.getState().setActiveFlow(flow);

      const state = useSelectionStore.getState();
      expect(state.activeFlow).toEqual(flow);
      expect(state.isFlowPlaying).toBe(true);
      expect(state.flowStep).toBe(0);
    });

    it("should clear active flow", () => {
      const flow: FlowDump = {
        id: "flow1",
        title: "Test Flow",
        steps: [],
      };

      useSelectionStore.getState().setActiveFlow(flow);
      useSelectionStore.getState().setActiveFlow(null);

      const state = useSelectionStore.getState();
      expect(state.activeFlow).toBeNull();
      expect(state.isFlowPlaying).toBe(false);
    });

    it("should set active requirement and clear others", () => {
      const flow: FlowDump = {
        id: "flow1",
        title: "Test Flow",
        steps: [],
      };

      useSelectionStore.getState().selectNode("Node1");
      useSelectionStore.getState().setActiveFlow(flow);
      useSelectionStore.getState().setActiveRequirement("Req1");

      const state = useSelectionStore.getState();
      expect(state.activeRequirement).toBe("Req1");
      expect(state.selectedNodeId).toBeNull();
      expect(state.activeFlow).toBeNull();
    });

    it("should set flow step", () => {
      useSelectionStore.getState().setFlowStep(5);
      expect(useSelectionStore.getState().flowStep).toBe(5);
    });

    it("should play flow", () => {
      useSelectionStore.getState().playFlow();
      expect(useSelectionStore.getState().isFlowPlaying).toBe(true);
    });

    it("should pause flow", () => {
      useSelectionStore.getState().playFlow();
      useSelectionStore.getState().pauseFlow();
      expect(useSelectionStore.getState().isFlowPlaying).toBe(false);
    });

    it("should go to next step", () => {
      const flow: FlowDump = {
        id: "flow1",
        title: "Test Flow",
        steps: [
          { id: "step1", title: "Step 1" },
          { id: "step2", title: "Step 2" },
          { id: "step3", title: "Step 3" },
        ],
      };

      useSelectionStore.getState().setActiveFlow(flow);
      useSelectionStore.getState().setFlowStep(0);

      useSelectionStore.getState().nextStep();
      expect(useSelectionStore.getState().flowStep).toBe(1);

      useSelectionStore.getState().nextStep();
      expect(useSelectionStore.getState().flowStep).toBe(2);

      useSelectionStore.getState().nextStep();
      expect(useSelectionStore.getState().flowStep).toBe(2); // Should not exceed max
    });

    it("should go to previous step", () => {
      useSelectionStore.getState().setFlowStep(2);

      useSelectionStore.getState().prevStep();
      expect(useSelectionStore.getState().flowStep).toBe(1);

      useSelectionStore.getState().prevStep();
      expect(useSelectionStore.getState().flowStep).toBe(0);

      useSelectionStore.getState().prevStep();
      expect(useSelectionStore.getState().flowStep).toBe(0); // Should not go below 0
    });
  });
});
