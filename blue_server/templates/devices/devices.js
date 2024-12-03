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

function renderTimeline(snapshot) {
	const ctx = document.getElementById('timelineChart').getContext('2d');

	const timeStart = snapshot.timeStart;
	const timeEnd = snapshot.timeEnd;

	const deviceData = snapshot.devices[0];

	const labels = deviceData.lifetime.map(step => {
		const start = new Date(step.timeStart);
		return `${start.toLocaleTimeString()} - ${step.distance}m`; // Customize the label format
	});

	const data = deviceData.lifetime.map(step => step.distance);

	new Chart(ctx, {
		type: 'bar', 
		data: {
			labels: labels,
			datasets: [{
				label: 'Distance to Device (m)',
				data: data,
				borderColor: 'rgba(75, 192, 192, 1)',
				backgroundColor: 'rgba(75, 192, 192, 0.2)',
				fill: true,
			}]
		},
		options: {
			responsive: true,
			scales: {
				x: {
					title: {
						display: true,
						text: 'Time',
					},
					ticks: {
						autoSkip: true,
						maxTicksLimit: 20,
					}
				},
				y: {
					title: {
						display: true,
						text: 'Distance (m)',
					}
				}
			}
		}
	});
}
