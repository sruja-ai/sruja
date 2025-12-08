---
title: "Lesson 1: Design a Video Streaming Service"
weight: 1
summary: "YouTube: Transcoding, CDNs, Adaptive Streaming."
---

# Lesson 1: Design a Video Streaming Service

**Goal:** Design a video sharing platform like YouTube or Netflix where users can upload and watch videos.

## Requirements

### Functional
*   Upload videos.
*   Watch videos (streaming).
*   Support multiple resolutions (360p, 720p, 1080p).

### Non-Functional
*   **Reliability:** No buffering.
*   **Availability:** Videos are always accessible.
*   **Scalability:** Handle millions of concurrent viewers.

## Core Design

### 1. Storage (Blob Store)
Videos are large binary files (BLOBs). Databases are bad for this.
*   **Object Storage:** AWS S3, Google Cloud Storage.
*   **Metadata:** Store title, description, and S3 URL in a SQL/NoSQL DB.

### 2. Processing (Transcoding)
Raw uploads are huge. We need to convert them into different formats and resolutions.
*   **Transcoding Service:** Breaks video into chunks and encodes them (H.264, VP9).

### 3. Delivery (CDN)
Serving video from a single server is too slow for global users.
*   **Content Delivery Network (CDN):** Caches video chunks in edge servers close to the user.

### 4. Adaptive Bitrate Streaming (HLS/DASH)
The player automatically switches quality based on the user's internet speed.

---

## 🛠️ Sruja Perspective: Modeling Infrastructure

We can use Sruja's `deployment` nodes to visualize the global distribution of content.

```sruja
architecture "Video Streaming" {
    system YouTube "Video Platform" {
        container WebApp "Web App"
        container API "API Server"
        
        container Transcoder "Transcoding Service" {
            description "Converts raw video to HLS format"
            scale { min 50 }
        }
        
        datastore S3 "Blob Storage" {
            description "Stores raw and processed video files"
        }
        
        datastore MetadataDB "Metadata DB"

        WebApp -> API "HTTPS"
        API -> MetadataDB "Reads/Writes"
        API -> S3 "Uploads"
        API -> Transcoder "Triggers"
        Transcoder -> S3 "Reads/Writes"
        Transcoder -> MetadataDB "Updates status"
    }

    // Deployment View
    deployment GlobalInfra "Global Infrastructure" {
        node OriginDC "Origin Data Center" {
            containerInstance WebApp
            containerInstance API
            containerInstance Transcoder
            containerInstance S3
        }
        
        node CDN "CDN (Edge Locations)" "Cloudflare / Akamai" {
            // Represents cached content
            node USEast "US-East Edge"
            node Europe "Europe Edge"
            node Asia "Asia Edge"
        }
    }

    person User "Viewer"

    // Streaming Flow
    scenario WatchVideo "User watches a video" {
        User -> WebApp "Get Video Page"
        WebApp -> API "Get Metadata (Title, URL)"
        API -> MetadataDB "Query"
        API -> User "Return Video Manifest URL"
        User -> CDN "Request Video Chunk (1080p)"
        CDN -> User "Stream Chunk"
    }

    // Upload Flow
    scenario UploadVideo "Creator uploads a video" {
        User -> API "Upload Raw Video"
        API -> S3 "Store Raw Video"
        API -> Transcoder "Trigger Transcoding Job"
        Transcoder -> S3 "Read Raw / Write HLS"
        Transcoder -> MetadataDB "Update Video Status"
    }
}
```
