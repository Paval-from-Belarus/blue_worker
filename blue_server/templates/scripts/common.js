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

async function fetchDeviceData() {
	try {
		const response = await fetch(`/api/v1/devices?start=${new Date()}&end=${new Date()}`);

		if (response.status == 200) {
			const deviceData = await response.json();
			renderTimeline(deviceData);
		} else {
			alert(`Server returns invalid status code = ${response.status}`)
		}

	} catch (error) {
		console.error('Error fetching device data:', error);
	}
}
