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
specification {
  element person
  element system
  element container
  element component
  element datastore
  element queue
}

model {
    YouTube = system "Video Platform" {
        WebApp = container "Web App"
        API = container "API Server"
        
        Transcoder = container "Transcoding Service" {
            description "Converts raw video to HLS format"
            scale { min 50 }
        }
        
        S3 = datastore "Blob Storage" {
            description "Stores raw and processed video files"
        }
        
        MetadataDB = datastore "Metadata DB"

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

    User = person "Viewer"

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
        User -> YouTube.WebApp "Upload Raw Video"
        YouTube.WebApp -> YouTube.API "POST /upload"
        YouTube.API -> YouTube.S3 "Store Raw Video"
        YouTube.API -> YouTube.Transcoder "Trigger Transcoding Job"
        YouTube.Transcoder -> YouTube.S3 "Read Raw / Write HLS"
        YouTube.Transcoder -> YouTube.MetadataDB "Update Video Status"
    }
    
    // Data flow: Video transcoding pipeline
    flow TranscodingPipeline "Video Transcoding Data Flow" {
        YouTube.S3 -> YouTube.Transcoder "Streams raw video chunks"
        YouTube.Transcoder -> YouTube.Transcoder "Encodes to HLS (360p, 720p, 1080p)"
        YouTube.Transcoder -> YouTube.S3 "Writes encoded chunks"
        YouTube.Transcoder -> YouTube.MetadataDB "Updates manifest URLs"
    }
    
    // Data flow: Video delivery pipeline
    flow DeliveryPipeline "Video Delivery Data Flow" {
        YouTube.S3 -> CDN "Replicates video chunks to edge"
        CDN -> User "Streams chunks on demand"
        User -> CDN "Requests next chunk based on bandwidth"
        CDN -> YouTube.S3 "Cache miss: fetch from origin"
    }
    
    // Data flow: Analytics pipeline
    flow AnalyticsPipeline "Video Analytics Data Flow" {
        YouTube.WebApp -> YouTube.API "Sends view events"
        YouTube.API -> YouTube.MetadataDB "Updates view count"
        YouTube.API -> YouTube.MetadataDB "Stores watch time"
        YouTube.MetadataDB -> YouTube.API "Aggregates analytics"
    }
}

views {
  view index {
    title "Complete Video Platform"
    include *
  }
  
  // Data flow view: Focus on data pipelines
  view dataflow {
    title "Data Flow View"
    include YouTube.Transcoder YouTube.S3 YouTube.MetadataDB
    exclude YouTube.WebApp YouTube.API
    description "Shows data transformation and storage flows"
  }
  
  // Processing view: Transcoding pipeline
  view processing {
    title "Processing Pipeline"
    include YouTube.Transcoder YouTube.S3
    exclude YouTube.WebApp YouTube.API YouTube.MetadataDB
    description "Focuses on video processing components"
  }
}
```
