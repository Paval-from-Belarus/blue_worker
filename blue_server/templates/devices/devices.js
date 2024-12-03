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
	const timeStep = 100; //min step in milliseconds

	const colors = ['rgba(75, 192, 192, 0.4)', 'rgba(255, 99, 132, 0.4)', 'rgba(255, 206, 86, 0.4)'];

	const datasets = [];

	snapshot.devices.forEach((deviceData) => {
		const data = deviceData.lifetime.flatMap(entry => {
			return [{
				x: entry.timeStart,
				y: deviceData.name,
				x1: entry.timeEnd,
				info: `MAC: ${deviceData.macAddress}\n Distance: ${entry.distance}`
			}, {
				x: entry.timeEnd,
				y: deviceData.name,
				info: `MAC: ${deviceData.macAddress}\n Distance: ${entry.distance}`
			}
			]
		});

		datasets.push({
			label: deviceData.name,
			data: data,
			borderColor: colors[0],
			backgroundColor: colors[1],
			fill: true,
		});
	});

	const scatterArbitraryLine = {
		id: 'scatterArbitraryLine',
		beforeDatasetsDraw(chart) {
			const { ctx,
				data: { datasets },
				scales: { x, y } } = chart;

			ctx.save();
			ctx.beginPath();

			ctx.lineWidth = 6;

			datasets.forEach(data => {
				data.data
					.filter(entry => entry.x1)
					.forEach(entry => {
						ctx.strokeStyle = data.backgroundColor;

						const startX = x.getPixelForValue(entry.x);
						const endX = x.getPixelForValue(entry.x1);

						const startY = y.getPixelForValue(entry.y);
						const endY = startY;

						ctx.moveTo(startX, startY);
						ctx.lineTo(endX, endY);
					});
			});


			//todo: add milestone label (for 15 minutes)

			ctx.stroke();
			ctx.closePath();
			ctx.restore();
		}
	};

	new Chart(ctx, {
		type: 'scatter',

		data: {
			datasets: datasets
		},

		options: {
			responsive: true,
			scales: {
				x: {
					title: {
						display: true,
						text: 'Time'
					},
				},
				y: {
					type: 'category', // Set y-axis as categorical
					title: {
						display: true,
						text: 'Devices'
					},
				}
			},
			plugins: {
				tooltip: {
					callbacks: {
						label: function(context) {
							const deviceInfo = context.dataset.data[context.dataIndex].info;
							return deviceInfo.split('\n');
						}
					}
				}
			}
		},

		plugins: [scatterArbitraryLine]
	});
}


