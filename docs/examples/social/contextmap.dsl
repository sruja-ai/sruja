contextMap {
  timeline.feed -> graph.followGraph: "upstream"
  timeline.feed -> media.mediaUpload: "customer-supplier"
  notifications.notify -> timeline.feed: "customer-supplier"
}