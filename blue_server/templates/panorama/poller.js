const imgElement = document.getElementById('image');
const pollingInterval = 100;

function updateImage() {
	const timestamp = new Date().getTime();
	imgElement.src = `panorama/frames?t=${timestamp}`;
}

// Start polling
setInterval(updateImage, pollingInterval);
