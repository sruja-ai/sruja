---
title: "Lesson 1: Interview Question - Design a Video Streaming Platform"
weight: 1
summary: "Master scaling questions by designing YouTube/Netflix-style systems."
---

# Lesson 1: Interview Question - Design a Video Streaming Platform

## The Interview Question

**"Design a video streaming platform like YouTube or Netflix that can handle millions of concurrent viewers."**

This is a **classic system design interview question** asked at Google, Netflix, and other top companies. Let's break it down step-by-step.

## Step 1: Clarify Requirements (What Interviewers Want to Hear)

Before jumping into design, **always clarify**:

**You should ask:**

- "What's the scale? How many concurrent viewers?"
- "What's the latency requirement? How fast should videos start?"
- "What types of videos? Short clips or full movies?"
- "Do we need to support live streaming or just on-demand?"

**Interviewer's typical answer:**

- "Let's say 10 million concurrent viewers"
- "Videos should start within 2 seconds"
- "Both short clips and full movies"
- "Focus on on-demand for now"

## Step 2: Design the High-Level Architecture

Start with the core components:

1. **Client** (mobile app, web browser)
2. **CDN** (Content Delivery Network) - serves videos
3. **Origin Server** - stores original videos
4. **API Server** - handles metadata, user requests
5. **Database** - stores video metadata, user data

## Step 3: Model with Sruja

Let's model this architecture:

```sruja
import { * } from 'sruja.ai/stdlib'


Viewer = person "Video Viewer"

StreamingPlatform = system "Video Streaming Service" {
CDN = container "Content Delivery Network" {
  technology "Cloudflare, AWS CloudFront"
  description "Serves videos from edge locations worldwide"
}

OriginServer = container "Origin Server" {
  technology "S3, GCS"
  description "Stores original video files"
}

VideoAPI = container "Video API" {
  technology "Go, gRPC"
  description "Handles video metadata, user requests"
}

TranscodingService = container "Video Transcoding" {
  technology "FFmpeg, Kubernetes"
  description "Converts videos to different formats/qualities"
}

VideoDB = database "Video Metadata Database" {
  technology "PostgreSQL"
}

UserDB = database "User Database" {
  technology "PostgreSQL"
}
}

Viewer -> StreamingPlatform.CDN "Streams video"
StreamingPlatform.CDN -> StreamingPlatform.OriginServer "Fetches on cache miss"
Viewer -> StreamingPlatform.VideoAPI "Requests video info"
StreamingPlatform.VideoAPI -> StreamingPlatform.VideoDB "Queries metadata"
StreamingPlatform.VideoAPI -> StreamingPlatform.UserDB "Queries user data"
StreamingPlatform.OriginServer -> StreamingPlatform.TranscodingService "Processes videos"

view index {
include *
}
```

## Step 4: Address Scaling (The Key Part)

**Interviewer will ask**: "How does this handle 10 million concurrent viewers?"

This is where you show your scaling knowledge. Let's add scaling configuration:

```sruja
import { * } from 'sruja.ai/stdlib'


Viewer = person "Video Viewer"

StreamingPlatform = system "Video Streaming Service" {
CDN = container "Content Delivery Network" {
  technology "Cloudflare, AWS CloudFront"
  // CDN scales automatically - no need to configure
  description "Serves videos from edge locations worldwide"
}

VideoAPI = container "Video API" {
  technology "Go, gRPC"

  // This is what interviewers want to see!
  scale {
    min 10
    max 1000
    metric "cpu > 75% or requests_per_second > 10000"
  }

  description "Handles video metadata, user requests"
}

TranscodingService = container "Video Transcoding" {
  technology "FFmpeg, Kubernetes"

  scale {
    min 5
    max 100
    metric "queue_length > 50"
  }

  description "Converts videos to different formats/qualities"
}

VideoDB = database "Video Metadata Database" {
  technology "PostgreSQL"
  // Database scaling: read replicas
  description "Primary database with 5 read replicas for scaling reads"
}
}

view index {
include *
}
```

## What Interviewers Look For

### ✅ Good Answer (What You Just Did)

1. **Clarified requirements** before designing
2. **Started with high-level** architecture
3. **Modeled with Sruja** to visualize
4. **Addressed scaling** with specific numbers
5. **Explained trade-offs** (CDN vs origin server)

### ❌ Bad Answer (Common Mistakes)

1. Jumping straight to code/implementation details
2. Not asking clarifying questions
3. Designing for small scale only
4. Not mentioning CDN or caching
5. Ignoring database scaling

## Key Points to Mention in Interview

### 1. CDN for Video Delivery

**Say**: "We use a CDN to serve videos from edge locations close to users. This reduces latency and offloads traffic from origin servers."

### 2. Horizontal Scaling for API

**Say**: "The API server scales horizontally from 10 to 1000 instances based on CPU and request rate. This handles traffic spikes during peak hours."

### 3. Database Read Replicas

**Say**: "We use read replicas for the database to scale read operations. Writes go to primary, reads can go to any replica."

### 4. Caching Strategy

**Say**: "We cache frequently accessed video metadata in Redis to reduce database load."

## Interview Practice: Add Caching

**Interviewer might ask**: "How do you reduce database load?"

Add caching to your design:

```sruja
import { * } from 'sruja.ai/stdlib'


StreamingPlatform = system "Video Streaming Service" {
VideoAPI = container "Video API" {
  technology "Go, gRPC"
  scale {
    min 10
    max 1000
    metric "cpu > 75%"
  }
}

VideoDB = database "Video Metadata Database" {
  technology "PostgreSQL"
}

Cache = database "Video Metadata Cache" {
  technology "Redis"
  description "Caches frequently accessed video metadata"
}
}

StreamingPlatform.VideoAPI -> StreamingPlatform.Cache "Reads metadata (cache hit)"
StreamingPlatform.VideoAPI -> StreamingPlatform.VideoDB "Reads metadata (cache miss)"
StreamingPlatform.VideoAPI -> StreamingPlatform.Cache "Writes to cache"

view index {
include *
}
```

## Understanding Scale Block Fields

### `min` - Minimum Replicas

**Interview tip**: "We keep at least 10 instances running to handle baseline traffic and provide fault tolerance."

### `max` - Maximum Replicas

**Interview tip**: "We cap at 1000 instances to control costs. If we need more, we'd need to optimize the architecture first."

### `metric` - Scaling Trigger

**Interview tip**: "We scale based on CPU usage and request rate. When CPU exceeds 75% or requests exceed 10k/sec, we add more instances."

## Real Interview Example: Capacity Estimation

**Interviewer**: "How many API servers do you need for 10M concurrent users?"

**Your answer**:

1. "Assume each user makes 1 request per minute = 10M requests/minute = ~167k requests/second"
2. "Each API server handles ~1000 requests/second"
3. "We need ~167 servers at peak"
4. "With 2x headroom for spikes: ~350 servers"
5. "Our scale block allows 10-1000, so we're covered"

## Exercise: Practice This Question

Design a video streaming platform and be ready to explain:

1. Why you chose CDN
2. How scaling works
3. Database scaling strategy
4. Caching approach

**Practice tip**: Time yourself (30-40 minutes) and explain out loud as if in an interview.

## Common Follow-Up Questions

Be prepared for:

- "How do you handle video uploads?" (Add upload service, queue for processing)
- "What about live streaming?" (Add live streaming infrastructure)
- "How do you ensure availability?" (Add redundancy, health checks)
- "What's the cost?" (Estimate based on scale)

## Next Steps

In the next lesson, we'll learn about SLOs (Service Level Objectives) - another common interview topic about defining performance targets.
