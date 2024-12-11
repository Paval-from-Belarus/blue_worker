GLOBAL_CHART = null;
GLOBAL_DEVICESS = null;

// Get the input element
const searchBar = document.getElementById("search-bar");

// Get the search results element
const searchResults = document.getElementById("search-results");

// Add an event listener for the input event on the search bar
searchBar.addEventListener("input", (event) => {
	// Get the search query from the event
	const query = event.target.value.toLowerCase();

	// Clear the search results
	searchResults.innerHTML = "";

	if (!GLOBAL_DEVICES) {
		alert("No devices");
	} else {
		for (const device of GLOBAL_DEVICES) {
			const macAddress = device.macAddress.toLowerCase();
			if (query.length > 0 && macAddress.startsWith(query)) {
				const listItem = document.createElement("li");

				listItem.textContent = macAddress;

				searchResults.appendChild(listItem);
			}
		}

	}

	searchResults.classList.add("active");
});


async function fetchDeviceData() {
	try {
		const response = await fetch('/api/v1/devices');

		if (response.status != 200) {
			alert(`Server returns invalid status code = ${response.status}`)
			return;
		}

		const snapshot = await response.json();
		const deviceListContainer = document.getElementById('device-list');
		const shouldFilter = document.getElementById('filter-valid-names').checked;

		snapshot.devices.forEach(device => {
			const label = document.createElement('label');
			const checkbox = document.createElement('input');
			checkbox.type = 'checkbox';
			checkbox.value = device.macAddress;
			checkbox.id = device.macAddress;

			checkbox.addEventListener('change', updateChart);
			const macDiv = document.createElement('span');
			macDiv.appendChild(document.createTextNode(device.macAddress));

			label.appendChild(checkbox);
			label.appendChild(macDiv);

			if (device.name.length >= 1) {
				const nameDiv = document.createElement('span');
				nameDiv.appendChild(document.createTextNode(` ${device.name}`));
				label.appendChild(nameDiv);
			}
			if (!shouldFilter) {
				deviceListContainer.appendChild(label);
			} else {
				if (device.name.length >= 1) {
					deviceListContainer.appendChild(label);
				}
			}

		});

		updateChart(snapshot);
	} catch (error) {
		alert('Error fetching device list', error);
	}
}

async function updateChart(snapshot) {
	const selectedMacs = Array
		.from(document.querySelectorAll('#device-list input[type="checkbox"]:checked'))
		.map(checkbox => checkbox.value);

	if (!snapshot) {
		response = await fetch('/api/v1/devices');

		if (response.status != 200) {
			alert(`Server returns invalid status code = ${response.status}`)
			return;
		}

		snapshot = await response.json();
	}

	GLOBAL_DEVICES = snapshot.devices;

	const devices = snapshot.devices
		.filter(device => selectedMacs.includes(device.macAddress));

	renderTimeline(devices)
}

function renderTimeline(devices) {
	const ctx = document.getElementById('timelineChart').getContext('2d');

	const colors = ['rgba(75, 192, 192, 0.4)', 'rgba(255, 99, 132, 0.4)', 'rgba(255, 206, 86, 0.4)'];

	const datasets = [];

	devices.forEach((deviceData, index) => {
		const data = deviceData.lifetime.flatMap(entry => {
			let deviceName = deviceData.name;
			if (deviceName && deviceName.length < 1) {
				deviceName = "<UNKNOWN>";
			}

			return [{
				x: entry.timeStart,
				y: deviceData.macAddress,
				x1: entry.timeEnd,
				info: `Name ${deviceName}\n Distance: ${entry.distance}`
			}, {
				x: entry.timeEnd,
				y: deviceData.macAddress,
				info: `Name: ${deviceName}\n Distance: ${entry.distance}`
			}
			]
		});

		datasets.push({
			label: deviceData.name,
			data: data,
			borderColor: colors[index % colors.length],
			backgroundColor: colors[index % colors.length],
			fill: true,
		});
	});

	if (GLOBAL_CHART) {
		const chart = GLOBAL_CHART;
		chart.data = { datasets: datasets };
		chart.update();
		return;
	}

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

	GLOBAL_CHART = new Chart(ctx, {
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
					ticks: {
						callback: function(value, index, ticks) {
							return formatTimestamp(value)
						},
						autoSkip: true,
						maxTicksLimit: 20
					}
				},
				y: {
					type: 'category',
					title: {
						display: true,
						text: 'Devices'
					},
				}
			},
			plugins: {
				legend: {
					display: false,
				},

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


function formatTimestamp(timestamp) {
	const date = new Date(timestamp);
	return `${date.getHours()} : ${date.getMinutes()}`
}

