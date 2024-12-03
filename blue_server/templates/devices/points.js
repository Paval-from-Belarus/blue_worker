// Scene, Camera, and Renderer Setup
const scene = new THREE.Scene();

const camera = new THREE.OrthographicCamera(
	window.innerWidth / -2, window.innerWidth / 2,
	window.innerHeight / 2, window.innerHeight / -2,
	1, 1000
);
camera.position.z = 10;

const renderer = new THREE.WebGLRenderer();
renderer.setSize(window.innerWidth, window.innerHeight);

document.body.appendChild(renderer.domElement);


// Helper to Create Concentric Circles
function createConcentricCircles(centerX, centerY, circleCount, spacing) {
	const circles = new THREE.Group();
	const material = new THREE.LineBasicMaterial({ color: 0x0000ff }); // Blue lines

	for (let i = 1; i <= circleCount; i++) {
		const geometry = new THREE.CircleGeometry(i * spacing, 64);
		const circle = new THREE.LineLoop(geometry, material);
		circle.position.set(centerX, centerY, 0);
		circles.add(circle);
	}

	return circles;
}

function generatePointCloud(scan) {
	const widgetRange = scan.farDistance * 1.2;
	const pixelsPerMetres = Math.min(window.innerWidth, window.innerHeight) / widgetRange;

	const positions = [];
	scan.points.forEach(point => {
		positions.push(point.x * pixelsPerMetres, point.y * pixelsPerMetres, 0);
	})

	const geometry = new THREE.BufferGeometry();
	geometry.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));

	const material = new THREE.PointsMaterial({ color: 0xff0000, size: 3 }); // Red points

	return new THREE.Points(geometry, material);
}

const POINT_CLOUD_SCENE_OBJECT_NAME = 'point_cloud';

function updatePositions() {
	fetch(`/api/v1/lidar-points/${LIDAR_ID}`)
		.then(response => {
			if (!response.ok) {
				throw new Error('Network response was not ok');
			}
			return response.json();
		})
		.then(scan => {
			const oldObject = scene.getObjectByName(POINT_CLOUD_SCENE_OBJECT_NAME);
			if (oldObject) {
				scene.remove(oldObject);
			}


			const pointCloud = generatePointCloud(scan);
			pointCloud.name = POINT_CLOUD_SCENE_OBJECT_NAME;

			scene.add(pointCloud);
		})
		.catch(error => console.error('Error fetching content:', error));

}

setInterval(updatePositions, 100);
// Animation Loop
function animate() {
	requestAnimationFrame(animate);
	renderer.render(scene, camera);
}
animate();

// Adjust on Window Resize
window.addEventListener('resize', () => {
	camera.left = window.innerWidth / -2;
	camera.right = window.innerWidth / 2;
	camera.top = window.innerHeight / 2;
	camera.bottom = window.innerHeight / -2;
	camera.updateProjectionMatrix();
	renderer.setSize(window.innerWidth, window.innerHeight);
});
