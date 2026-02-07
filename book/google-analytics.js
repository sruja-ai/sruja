/**
 * Google Analytics 4 (GA4) for mdBook.
 * Set window.MDBOOK_GA_MEASUREMENT_ID before this script runs, or replace
 * the placeholder below with your GA4 Measurement ID (e.g. G-XXXXXXXXXX).
 * If unset or placeholder, no tracking runs.
 */
(function () {
  var id = window.MDBOOK_GA_MEASUREMENT_ID || 'G-SC9EM616YS';
  if (!id) return;

  window.dataLayer = window.dataLayer || [];
  function gtag() { dataLayer.push(arguments); }
  window.gtag = gtag;
  gtag('js', new Date());
  gtag('config', id, { anonymize_ip: true });

  var s = document.createElement('script');
  s.async = true;
  s.src = 'https://www.googletagmanager.com/gtag/js?id=' + id;
  document.head.appendChild(s);
})();
