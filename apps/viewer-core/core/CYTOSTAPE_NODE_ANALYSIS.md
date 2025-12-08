# Cytoscape Node Sizing Analysis

## How Cytoscape Handles `width: 'label'`

When you set `width: 'label'`:
1. Cytoscape measures the TEXT width (not including padding or icons)
2. Sets node width = text width
3. Applies padding INSIDE the node
4. Positions text starting from padding edge
5. Background images are positioned relative to node bounds

## The Problem

For a node with:
- Icon at left (background-position-x: 12px, width: 16px) → icon spans 12px to 28px
- Text: "Web Interface"
- padding-left: 40px
- text-halign: 'left'

What happens:
- Text width is calculated (e.g., 100px for "Web Interface")
- Node width = 100px
- Padding-left = 40px is applied
- Text starts at 40px from left edge
- BUT icon is at 12px, which is INSIDE the text area (0-100px)

## The Solution

The padding-left must be >= (icon position + icon width + gap)

For Container:
- Icon at: 12px
- Icon width: 16px  
- Icon ends at: 28px
- Need gap: 8-12px
- padding-left should be: 36-40px minimum
- Text starts at: padding-left = 40px (safe)

But we also need text-margin-x to push text further right if needed.

## Best Practice Approach

1. **Calculate icon space**: icon_position + icon_width + gap
2. **Set padding-left** to that value
3. **Set text-margin-x** to push text even further if needed
4. **Use explicit min-width** to ensure nodes aren't too small
5. **Reduce text-max-width** to prevent overflow

## Alternative: Use Explicit Width

Instead of `width: 'label'`, calculate:
- width = text_width + padding_left + padding_right
- But this requires measuring text, which is complex

## Recommended Fix

1. Ensure padding-left > (icon_x + icon_width + 8px gap)
2. Use text-margin-x to add extra space
3. Set reasonable min-width
4. Reduce text-max-width for wrapping

