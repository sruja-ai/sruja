contextMap {
  messaging.core -> channels.groups: "upstream"
  messaging.core -> search.index: "customer-supplier"
  media.mediaStore -> messaging.core: "customer-supplier"
}