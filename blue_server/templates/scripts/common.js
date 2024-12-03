"use strict";
const convert = (serverUri) => {
	switch (serverUri.toLowerCase()) {
		case 'lidar': return '/lidar';
		case 'panorama': return '/panorama';
		case 'qper': return 'https://qper.rin-q.by';
		default: return "/"
	}
}
const changePage = (uri) => {
	let file = convert(uri);
	window.location.replace(file);
}
const setPage = (uri) => {
	window.location.replace(uri);
}
const useExternalSources = (input) => {
	console.log("Vivate le opene source!");
}