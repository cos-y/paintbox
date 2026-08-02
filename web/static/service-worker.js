/// <reference lib="webworker" />

const CACHE = 'paintbox-v1';

const PRE_CACHE = ['/search/', '/stock/', '/gamut/', '/about/', '/manifest.json'];

self.addEventListener('install', (e) => {
	e.waitUntil(
		caches.open(CACHE).then((cache) => cache.addAll(PRE_CACHE))
	);
	self.skipWaiting();
});

self.addEventListener('activate', (e) => {
	e.waitUntil(
		caches.keys().then((keys) =>
			Promise.all(keys.filter((k) => k !== CACHE).map((k) => caches.delete(k)))
		)
	);
	self.clients.claim();
});

self.addEventListener('fetch', (e) => {
	if (e.request.method !== 'GET') return;
	e.respondWith(
		caches.match(e.request).then((cached) => cached ?? fetch(e.request))
	);
});
