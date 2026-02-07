# Sruja Onboarding Transformation Plan

> **Goal:** Enable any user—from student to senior engineer—to create their first successful diagram in under 5 minutes.

---

## 📋 Executive Summary

Sruja is a powerful architecture-as-code tool, but its current UX is optimized for power users. This plan transforms onboarding to be **Product-Led**, prioritizing immediate value over comprehensive learning.

**Core Strategy:** Reduce friction first, add features second.

---

## 🔴 Current State: Key Pain Points

| Pain Point             | Impact                                          | Priority |
| :--------------------- | :---------------------------------------------- | :------- |
| **Cognitive Overload** | 6 tabs visible; users don't know where to start | P0       |
| **"Kinds" Barrier**    | Manual `kind` declarations confuse everyone     | P0       |
| **Unclear Value Prop** | Blank canvas doesn't show what's possible       | P1       |
| **Sync Confusion**     | Visual ↔ Code source of truth is ambiguous      | P1       |
| **Technical Errors**   | `Invalid reference` errors lack guidance        | P2       |

---

## ✅ Solution: Product-Led Onboarding

### Phase 1: Quick Wins (Week 1-2)

#### 1.1 Smart Defaults

**Problem:** Users must manually declare `person = kind "Person"` before creating anything.

**Solution:** Auto-import `stdlib` for new files OR pre-populate with common kinds.

```sruja
// New files start with this automatically:
import { * } from 'sruja.ai/stdlib'

// User can immediately write:
user = person "User"
app = system "My App"
```

**Implementation:**

- Modify `apps/designer/src/stores/editorStore.ts` to inject default import on new file.
- No backend changes required.

---

#### 1.2 Single-Tab Start Mode

**Problem:** 6 tabs (Overview, Diagram, Code, Builder, Details, Roles) overwhelm new users.

**Solution:** New users see only **Canvas + Code Preview**. Other tabs unlock progressively.

```
┌─────────────────────────────────────────────────────────┐
│  [Diagram]  [Code]  ▸ More...                         │
├─────────────────────────────────────────────────────────┤
│                                                         │
│              [Visual Canvas]                           │
│                                                         │
├─────────────────────────────────────────────────────────┤
│  // Live Code Preview                                  │
│  user = person "User"                                  │
│  app = system "My App"                                 │
└─────────────────────────────────────────────────────────┘
```

**Implementation:**

- Add `beginnerMode` flag to `apps/designer/src/stores/viewStore.ts`.
- Conditionally render tabs based on flag.

---

#### 1.3 Guided Onboarding Tour

**Problem:** Users don't know where to start on a blank canvas.

**Solution:** 3-step interactive tour triggered on first visit.

| Step | Action               | Prompt                            |
| :--- | :------------------- | :-------------------------------- |
| 1    | Click "Add System"   | "Let's add your first system!"    |
| 2    | Click "Add Database" | "Great! Now add a database."      |
| 3    | Drag to connect      | "Connect them to show data flow." |

**Implementation:**

- Use a library like `react-joyride` or custom overlay.
- Store completion in `LocalStorage`.

---

### Phase 2: Error & Help Improvements (Week 3-4)

#### 2.1 Contextual Error Messages

**Before:**

```
Error: Invalid reference "User" at line 15
```

**After:**

```
⚠️ Couldn't find "User" (line 15)

Available elements:
  - user (lowercase)

💡 Did you mean: user?  [Fix it for me]
```

**Implementation:**

- Extend error/suggestion logic in the Rust/TS LSP layer.
- Expose suggestions via LSP hover/quickfix.

---

#### 2.2 "What's This?" Tooltips

Add contextual help icons next to key concepts (Kinds, Containers, Views).

**Implementation:**

- Add HelpTooltip in the VS Code extension or shared UI package.
- Map keywords to explanations in a config file.

---

### Phase 3: Lightweight Gamification (Month 2)

#### 3.1 LocalStorage Progress Tracking

Track simple milestones without requiring login.

```typescript
// apps/designer/src/stores/progressStore.ts
interface Progress {
  firstSystemCreated: boolean;
  firstConnectionMade: boolean;
  tutorialCompleted: boolean;
}
```

**UI:** Simple progress indicator (e.g., "3/5 steps complete").

---

### Phase 4: Content & Marketing (Month 3+)

Only after core UX is validated:

- Write "System Design Zero to One" course.
- Create 3 beginner examples.
- Produce 3 x 5-min video tutorials.

---

## 📊 Success Metrics

| Metric                | Current   | Target (3 months) |
| :-------------------- | :-------- | :---------------- |
| Time to first diagram | 10-60 min | < 5 min           |
| Tutorial completion   | Unknown   | > 60%             |
| User retention (24h)  | Unknown   | > 30%             |
| Error resolution time | 2-5 min   | < 30s             |

---

## 🗓️ Implementation Timeline

| Week | Deliverable                                      |
| :--- | :----------------------------------------------- |
| 1-2  | Smart Defaults, Single-Tab Mode, Onboarding Tour |
| 3-4  | Error Messages, Tooltips                         |
| 5-6  | LocalStorage Progress, UI Polish                 |
| 7-8  | Beta Testing, Iteration                          |
| 9-12 | Content Creation, Launch                         |

---

## 💰 Resource Estimate (Reduced Scope)

| Role               | Allocation                   |
| :----------------- | :--------------------------- |
| Frontend Dev       | 100% (6 weeks)               |
| Backend Dev        | 10% (1 week, error messages) |
| Content Writer     | 50% (4 weeks, Month 3)       |
| **Estimated Cost** | **~$30,000**                 |

---

## 🎯 Key Decisions

1.  **No Backend for Gamification:** V1 uses LocalStorage. Login/persistence deferred.
2.  **No Mobile Optimization:** Focus on desktop experience.
3.  **No AI Features (V1):** "Explain Like I'm 5" deferred to V2.
4.  **Content Last:** Courses and videos only after core UX is proven.

---

## ✅ Immediate Next Steps

1.  **[ ] Implement Smart Defaults** in `editorStore.ts`.
2.  **[ ] Add `beginnerMode` toggle** to `viewStore.ts`.
3.  **[ ] Integrate `react-joyride`** for onboarding tour.
4.  **[ ] Improve top 5 error messages** in `pkg/dx`.
