// apps/designer/src/components/Product/UserStoryCanvas.tsx
import { useState } from "react";
import { Plus, User, FileText, X } from "lucide-react";
import { Button, Input } from "@sruja/ui";
import "./UserStoryCanvas.css";

interface UserStory {
  id: string;
  story: string; // "As a [user], I want [goal] so that [benefit]"
  mappedComponents: string[]; // Component IDs this story maps to
}

interface UserStoryCanvasProps {
  onStoryMap?: (story: UserStory, componentId: string) => void;
}

export function UserStoryCanvas({ onStoryMap: _onStoryMap }: UserStoryCanvasProps) {
  const [stories, setStories] = useState<UserStory[]>([]);
  const [newStory, setNewStory] = useState("");
  const [showAddForm, setShowAddForm] = useState(false);

  const handleAddStory = () => {
    if (!newStory.trim()) return;

    const story: UserStory = {
      id: `story-${Date.now()}`,
      story: newStory.trim(),
      mappedComponents: [],
    };

    setStories([...stories, story]);
    setNewStory("");
    setShowAddForm(false);
  };

  const handleDeleteStory = (id: string) => {
    setStories(stories.filter((s) => s.id !== id));
  };

  return (
    <div className="user-story-canvas">
      <div className="user-story-canvas-header">
        <h3 className="user-story-canvas-title">
          <User size={18} />
          User Stories
        </h3>
        <p className="user-story-canvas-subtitle">Map user stories to architecture components</p>
      </div>

      <div className="user-story-canvas-content">
        {!showAddForm ? (
          <Button
            variant="secondary"
            size="sm"
            onClick={() => setShowAddForm(true)}
            className="add-story-button"
          >
            <Plus size={16} />
            Add User Story
          </Button>
        ) : (
          <div className="add-story-form">
            <div className="add-story-input-wrapper">
              <FileText size={16} className="story-icon" />
              <Input
                type="text"
                placeholder="As a [user], I want [goal] so that [benefit]"
                value={newStory}
                onChange={(e) => setNewStory(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") {
                    handleAddStory();
                  } else if (e.key === "Escape") {
                    setShowAddForm(false);
                    setNewStory("");
                  }
                }}
                className="story-input"
                autoFocus
              />
            </div>
            <div className="add-story-actions">
              <Button variant="primary" size="sm" onClick={handleAddStory}>
                Add
              </Button>
              <Button
                variant="ghost"
                size="sm"
                onClick={() => {
                  setShowAddForm(false);
                  setNewStory("");
                }}
              >
                Cancel
              </Button>
            </div>
          </div>
        )}

        <div className="user-stories-list">
          {stories.length === 0 ? (
            <div className="user-stories-empty">
              <p>No user stories yet.</p>
              <p className="user-stories-empty-subtitle">
                Add user stories to map them to architecture components.
              </p>
            </div>
          ) : (
            stories.map((story) => (
              <div key={story.id} className="user-story-item">
                <div className="user-story-item-header">
                  <div className="user-story-text">
                    <User size={14} className="user-icon" />
                    {story.story}
                  </div>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => handleDeleteStory(story.id)}
                    className="delete-story-button"
                    aria-label="Delete story"
                  >
                    <X size={14} />
                  </Button>
                </div>
                {story.mappedComponents.length > 0 && (
                  <div className="user-story-mapped">
                    Mapped to: {story.mappedComponents.join(", ")}
                  </div>
                )}
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}
