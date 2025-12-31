# Demo Video Production Guide

## 15-Second Bidirectional Sync Showcase

**Goal**: Show Sruja's killer feature - bidirectional sync between canvas and code

**Target audience**: Developers, architects, product managers  
**Platform**: LinkedIn, Twitter, YouTube, website hero

---

## Video Script (15 seconds)

### Storyboard

| Time   | Visual                                              | Action                                                            | Narration/Text Overlay                                         |
| ------ | --------------------------------------------------- | ----------------------------------------------------------------- | -------------------------------------------------------------- |
| 0-3s   | Designer app - Canvas view with simple architecture | Cursor adds new "API Gateway" component by dragging               | _"Add component visually..."_                                  |
| 3-6s   | Switch to Code tab                                  | Show DSL code, **highlight new component** appearing in real-time | _"...code updates automatically"_                              |
| 6-9s   | Code tab - cursor editing DSL                       | Add new relationship: `user -> gateway "requests"`                | _"Edit code..."_                                               |
| 9-12s  | Switch back to Canvas                               | **Show new arrow** appearing on diagram                           | _"...diagram updates"_                                         |
| 12-15s | Split screen: Canvas + Code side-by-side            | Both update simultaneously                                        | **"Bidirectional Sync"**<br>**"Like Notion for Architecture"** |

---

## Recording Instructions

### Setup (Before Recording)

**1. Prepare Example Architecture**:

```sruja
specification {
  element person
  element system
  element container
}

model {
  user = person "User" {
    description "Application user"
  }

  app = system "E-Commerce App" {
    web = container "Web App" {
      technology "React"
    }
    db = container "Database" {
     technology "PostgreSQL"
    }
  }

  user -> app.web "browses"
  app.web -> app.db "reads/writes"
}
```

**2. Browser Setup**:

- Load Designer app: http://localhost:5173 (or deployed URL)
- Import the example above
- Set zoom to 100% for clarity
- Position canvas to show all elements clearly

**3. Recording Tools**:

- **Mac**: QuickTime (Cmd+Shift+5) or OBS
- **Windows**: OBS or Xbox Game Bar
- **Linux**: OBS or SimpleScreenRecorder

**Settings**:

- Resolution: 1920x1080 (Full HD)
- Frame rate: 30fps minimum (60fps better)
- Audio: Off (will add text overlays)
- Cursor: Visible and large enough to see

---

## Step-by-Step Recording

### Segment 1 (0-3s): Add Component Visually

**Setup**:

1. Have Canvas view open
2. Cursor positioned near "Add Component" or ready to drag

**Actions**:

1. Click "Add Component" or open component palette
2. Drag "API Gateway" component
3. Place between User and Web App
4. Name it "API Gateway"
5. **Pause briefly** (for next cut)

**Camera tip**: Keep cursor movements smooth and deliberate

---

### Segment 2 (3-6s): Code Updates

**Setup**:

1. Immediately after adding component
2. Cursor ready to click "Code" tab

**Actions**:

1. Click "Code" tab
2. **Scroll to show new component** in DSL:
   ```sruja
   gateway = container "API Gateway" {
     technology "Kong"
   }
   ```
3. **Highlight this section** (draw box or use cursor to point)
4. **Pause briefly**

**Camera tip**: Use animated highlight or cursor to draw attention to new code

---

### Segment 3 (6-9s): Edit Code

**Setup**:

1. Code tab still open
2. Cursor positioned in DSL editor

**Actions**:

1. Scroll to relationships section
2. **Type new relationship**:
   ```sruja
   user -> gateway "sends requests"
   gateway -> app.web "forwards"
   ```
3. **Pause for 1s** (let debounce trigger)
4. **Show "Syncing..." indicator** (if visible)

**Camera tip**: Type at normal speed (don't rush), show the sync indicator

---

### Segment 4 (9-12s): Diagram Updates

**Setup**:

1. Code still showing new relationship
2. Cursor ready to switch tabs

**Actions**:

1. Wait for sync to complete ("Synced" indicator)
2. Click "Canvas" tab
3. **Camera zooms slightly** to show new arrows
4. **Highlight new relationships** with cursor or animation
5. **Pause briefly**

**Camera tip**: Use smooth transition, highlight the changes clearly

---

### Segment 5 (12-15s): Side-by-Side + Branding

**Setup**:

1. Prepare split-screen view (or edit in post)

**Actions**:

1. Show Canvas on left, Code on right (split screen)
2. Make small edit on Canvas (move a component)
3. Show Code updating
4. **Add text overlay**:
   - "Bidirectional Sync"
   - "Like Notion for Architecture"
5. **Add logo/CTA**: "Try Sruja Designer"

**Camera tip**: This segment can be created in editing

---

## Editing Guide

### Tools

- **Simple**: iMovie (Mac), Clipchamp (Windows)
- **Advanced**: DaVinci Resolve (free), Premiere Pro

### Editing Steps

**1. Cut and Arrange** (5 segments)

- Import all recordings
- Cut to 3 seconds each
- Total: 15 seconds

**2. Add Text Overlays**

- 0-3s: "Add component visually..."
- 3-6s: "...code updates automatically"
- 6-9s: "Edit code..."
- 9-12s: "...diagram updates"
- 12-15s: "Bidirectional Sync" + "Like Notion for Architecture"

**Text style**:

- Font: Inter, Roboto, or SF Pro
- Size: Large (72pt minimum)
- Color: White with dark semi-transparent background
- Position: Lower third
- Animation: Fade in (0.3s)

**3. Add Highlights/Annotations**

- Draw attention boxes (red or yellow)
- Cursor zoom effect
- Arrow animations
- Use tools: Camtasia, ScreenFlow, or Kapwing

**4. Add Sync Indicator Animation**

- Show "Syncing..." → "Synced" transition
- Can overlay manually if not visible in recording

**5. Final Frame (14-15s)**

- Logo: Sruja logo (top left)
- CTA: "Try Sruja Designer" (center)
- URL: designer.sruja.ai (bottom)

**6. Export Settings**

- Format: MP4 (H.264)
- Resolution: 1920x1080
- Frame rate: 30fps
- Bitrate: 8-10 Mbps
- Audio: None (or add subtle background music)

---

## Alternative: Screen Recording with Browser DevTools

If you want perfect split-screen, record using browser developer tools:

**Setup**:

1. Open Designer
2. Open DevTools (F12)
3. Dock DevTools to right (Cmd+Shift+D)
4. Resize so Canvas is on left, Code panel visible on right

**Record**:

- Both views visible simultaneously
- Make edits, show sync in real-time
- No post-editing split-screen needed

---

## Quick Version (If Short on Time)

**Simplified 10-second version**:

1. **0-4s**: Canvas view → Add component
2. **4-6s**: Fast transition to Code tab, show new code
3. **6-8s**: Fast transition back to Canvas
4. **8-10s**: Text overlay: "Bidirectional Sync - Try Sruja"

---

## Checklist

**Before Recording**:

- [ ] Designer app loaded with example
- [ ] Canvas zoomed appropriately
- [ ] Recording software tested (OBS/QuickTime)
- [ ] Cursor visible and large
- [ ] Browser at 100% zoom

**During Recording**:

- [ ] Smooth cursor movements
- [ ] Deliberate pauses between actions
- [ ] Sync indicators visible
- [ ] Changes clearly visible

**During Editing**:

- [ ] 15 seconds total (or 10-second quick version)
- [ ] Text overlays readable
- [ ] Highlights draw attention to changes
- [ ] Final frame has logo + CTA
- [ ] Export at 1080p 30fps

**Before Publishing**:

- [ ] Preview on mobile (text readable?)
- [ ] Test on different platforms (LinkedIn, Twitter)
- [ ] Upload to YouTube (unlisted first)
- [ ] Add to website hero

---

## Publishing Checklist

**YouTube**:

- Title: "Sruja - Bidirectional Sync Demo"
- Description: "Edit architecture visually or in code - changes sync both ways. Try Sruja Designer: designer.sruja.ai"
- Tags: architecture, software design, visual editor, code sync
- Thumbnail: Frame 12s (split screen with text)

**LinkedIn**:

- Upload as native video (better reach)
- Caption: "We built bidirectional sync for architecture diagrams. Edit visually or in code - changes sync automatically. Try it: [link]"

**Twitter**:

- Upload directly (not YouTube link)
- Tweet: "Visual + Code sync for architecture diagrams.\n\nEdit either way → Changes sync automatically.\n\nLike Notion, but for architecture.\n\nTry it: [link]"

**Website**:

- Embed on hero section
- Autoplay (muted) on page load
- Loop continuously

---

## Example Timeline (Shot List)

```
00:00 - Fade in: Canvas view with existing architecture
00:01 - Cursor moves to add component
00:02 - Component appears ("API Gateway")
00:03 - Quick transition: Click "Code" tab
00:04 - Code view appears, scroll to new component
00:05 - Highlight/box around new code
00:06 - Transition out, cursor to editor
00:07 - Start typing new relationship
00:08 - Finish typing, "Syncing..." appears
00:09 - Transition: Click "Canvas" tab
00:10 - Canvas appears, new arrows visible
00:11 - Cursor highlights new arrows
00:12 - Split screen: Canvas left, Code right
00:13 - Text overlay: "Bidirectional Sync"
00:14 - Text overlay: "Like Notion for Architecture"
00:15 - Final frame: Logo + "Try Sruja Designer"
```

---

## Tips for Success

1. **Record multiple takes** - You'll get smoother on take 3-4
2. **Keep cursor movements slow** - Easier to follow
3. **Pause between actions** - Gives time to process
4. **Test on mobile** - Ensure text is readable on small screens
5. **Add subtle zoom** - Draw attention to specific areas
6. **Use brand colors** - Purple/blue gradients for overlays

---

**Time estimate**:

- Recording: 30-45 mins (including setup and retakes)
- Editing: 45-60 mins (first time)
- Total: 90 mins for high-quality result

**Quick version**: 30 mins total (minimal editing)
