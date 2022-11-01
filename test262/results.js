const formatter = new Intl.NumberFormat("en-GB");

loadMainData();
loadMainResults();

const response = await fetch(
  "https://api.github.com/repos/boa-dev/boa/releases"
);
const releases = await response.json();
releases.sort((a, b) => compareVersions(a.tag_name, b.tag_name) * -1);
const latestTag = releases[0].tag_name;
loadLatestVersionResults(latestTag);

const releaseTags = [];
for (const release of releases) {
  const tag = release.tag_name;
  const version = tag.split(".");

  // We know there is no data for versions lower than v0.10.
  if (version[0] == "v0" && parseInt(version[1]) < 10) {
    continue;
  }

  releaseTags.push(tag);
}

const releaseData = new Map();

const versionListHTMLItems = await Promise.all(
  releaseTags.map(async (tag) => {
    const response = await fetch(`./refs/tags/${tag}/latest.json`);
    const json = await response.json();

    releaseData.set(tag, json);

    return `<li class="list-group-item d-flex justify-content-between">
      <div class="d-flex align-items-center gap-1">
        <b>${tag}</b>
        <span class="text-success">${formatter.format(json.r.o)}</span>
        /
        <span class="text-warning">${formatter.format(json.r.i)}</span>
        /
        <span class="text-danger">${formatter.format(
          json.r.c - json.r.o - json.r.i
        )}
        ${
          json.r.p !== 0
            ? ` (${formatter.format(
                json.r.p
              )} <i class="bi-exclamation-triangle"></i>)`
            : ""
        }</span>
        /
        <b>${formatter.format(
          Math.round((10000 * json.r.o) / json.r.c) / 100
        )}%</b>
      </div>
      <button type="button" class="btn btn-outline-primary" id="old-version-${tag}">
        Test Results
      </button>
    </li>`;
  })
);

document.getElementById("old-versions-list").innerHTML =
  versionListHTMLItems.join("");

document
  .getElementById("latest-version-info-link")
  .addEventListener("click", () => {
    showData(releaseData.get(latestTag));
  });

releaseData.forEach((data, tag) => {
  document
    .getElementById(`old-version-${tag}`)
    .addEventListener("click", () => {
      showData(data);
    });
});

async function loadMainData() {
  const response = await fetch("./refs/heads/main/latest.json");
  const data = await response.json();

  document.getElementById("main-info-link").addEventListener("click", () => {
    showData(data);
  });
}

async function loadMainResults() {
  const response = await fetch("./refs/heads/main/results.json");
  const data = await response.json();

  createInfoFromResults(data, "main-results");

  new Chart(document.getElementById("main-graph"), {
    type: "line",
    data: {
      labels: data.map((data) => data.c),
      datasets: [
        {
          label: "Passed",
          data: data.map((data) => data.o),
          backgroundColor: "#1fcb4a",
          borderColor: "#0f6524",
          borderWidth: 1,
          fill: true,
        },
        {
          label: "Ignored",
          data: data.map((data) => data.i),
          backgroundColor: "#dfa800",
          borderColor: "#6f5400",
          borderWidth: 1,
          fill: true,
        },
        {
          label: "Panics",
          data: data.map((data) => data.p),
          backgroundColor: "#a30000",
          borderColor: "#510000",
          borderWidth: 1,
          fill: true,
        },
        {
          label: "Failed",
          data: data.map((data) => data.t - data.i - data.o - data.p),
          backgroundColor: "#ff4848",
          borderColor: "#a30000",
          borderWidth: 1,
          fill: true,
        },
      ],
    },
    options: {
      elements: {
        point: {
          radius: 0,
        },
      },
      legend: {
        display: true,
      },
      responsive: true,
      tooltips: {
        mode: "index",
      },
      interaction: {
        mode: "nearest",
        axis: "x",
        intersect: false,
      },
      scales: {
        x: {
          display: false,
          title: {
            display: false,
          },
        },
        y: {
          stacked: true,
          title: {
            display: true,
            text: "Tests",
          },
        },
      },
    },
  });
}

async function loadLatestVersionResults(tag) {
  const response = await fetch(`./refs/tags/${tag}/results.json`);
  const data = await response.json();

  createInfoFromResults(data, "latest-version-results");
}

function createInfoFromResults(resultsData, nodeID) {
  const latest = resultsData[resultsData.length - 1];

  document.getElementById(nodeID).insertAdjacentHTML(
    "afterbegin",
    `
    <li class="list-group-item">
      Latest commit: <a href="https://github.com/boa-dev/boa/commit/${
        latest.c
      }" title="Check commit">${latest.c}</a>
    </li>
    <li class="list-group-item">
      Total tests: <span>${formatter.format(latest.t)}</span>
    </li>
    <li class="list-group-item">
      Passed tests: <span class="text-success">${formatter.format(
        latest.o
      )}</span>
    </li>
    <li class="list-group-item">
      Ignored tests: <span class="text-warning">${formatter.format(
        latest.i
      )}</span>
    </li>
    <li class="list-group-item">
      Failed tests: <span class="text-danger">${formatter.format(
        latest.t - latest.o - latest.i
      )}
      ${
        latest.p !== 0
          ? ` (${formatter.format(
              latest.p
            )} <i class="bi-exclamation-triangle"></i>)`
          : ""
      }</span>
    </li>
    <li class="list-group-item">
      Conformance: <b>${Math.round((10000 * latest.o) / latest.t) / 100}%</b>
    </li>
  `
  );
}

// Shows the full test data.
function showData(data) {
  const infoContainer = document.getElementById("info");
  const progressInfoContainer = document.getElementById("progress-info");

  const totalTests = data.r.c;
  const passedTests = data.r.o;
  const ignoredTests = data.r.i;
  const failedTests = totalTests - passedTests - ignoredTests;

  infoContainer.innerHTML = "";
  progressInfoContainer.innerHTML = `<div class="progress g-0">
    <div
      class="progress-bar progress-bar bg-success"
      aria-valuenow="${passedTests}"
      aria-valuemax="${totalTests}"
      aria-valuemin="0"
      role="progressbar"
      style="width: ${Math.round((passedTests / totalTests) * 10000) / 100}%;"
    ></div>
    <div
      class="progress-bar progress-bar bg-warning"
      aria-valuenow="${ignoredTests}"
      aria-valuemax="${totalTests}"
      aria-valuemin="0"
      role="progressbar"
      style="width: ${Math.round((ignoredTests / totalTests) * 10000) / 100}%;"
    ></div>
    <div
      class="progress-bar progress-bar bg-danger"
      aria-valuenow="${failedTests}"
      aria-valuemax="${totalTests}"
      aria-valuemin="0"
      role="progressbar"
      style="width: ${Math.round((failedTests / totalTests) * 10000) / 100}%;"
    ></div>
  </div>`;

  for (const suite of data.r.s) {
    addSuite(suite, "info", "test/" + suite.n, data.u);
  }
}

function addSuite(suite, parentID, namespace, upstream) {
  const newID = parentID + suite.n;
  const newInnerID = newID + "-inner";
  const headerID = newID + "header";

  const html = `<div class="accordion-item">
    <h2 id="${headerID}" class="accordion-header">
      <button
        type="button"
        class="accordion-button"
        aria-expanded="false"
        data-bs-toggle="collapse"
        aria-controls="${newID}"
        data-bs-target="#${newID}"
      >
        <span class="data-overview">
          <span class="name">${suite.n}</span>
          <span class="text-success">${formatter.format(suite.o)}</span>
          /
          <span class="text-warning">${formatter.format(suite.i)}</span>
          /
          <span class="text-danger">${formatter.format(
            suite.c - suite.o - suite.i
          )}
          ${
            suite.p !== 0
              ? ` (${formatter.format(
                  suite.p
                )} <i class="bi-exclamation-triangle"></i>)`
              : ""
          }</span>
          /
          <span>${formatter.format(suite.c)}</span>
        </span>
      </button>
    </h2>
    <div id="${newID}" class="accordion-collapse collapse" aria-labelledby="${headerID}" data-bs-parent="#${parentID}">
      <div id="${newInnerID}" class="accordion-body">
      </div>
    </div>
  </div>`;

  document.getElementById(parentID).insertAdjacentHTML("beforeend", html);

  const newContainer = document.getElementById(newID);
  const newInnerContainer = document.getElementById(newInnerID);

  newContainer.addEventListener("show.bs.collapse", (event) => {
    event.stopPropagation();

    if (typeof suite.t !== "undefined" && suite.t.length !== 0) {
      const rows = suite.t.map((innerTest) => {
        const panics = innerTest.r === "P";
        let style;
        switch (innerTest.r) {
          case "O":
            style = "bg-success";
            break;
          case "I":
            style = "bg-warning";
            break;
          default:
            style = "bg-danger";
        }

        return `<a
          title="${innerTest.n}"
          class="card test embed-responsive ${style}${
          panics ? "" : " embed-responsive-1by1"
        }"
          target="_blank"
          href="https://github.com/tc39/test262/blob/${upstream}/${namespace}/${
          innerTest.n
        }.js"
        >${panics ? '<i class="bi-exclamation-triangle"></i>' : ""}</a>`;
      });

      const testsHTML = `<div class="card">
        <div class="row card-body">
          <h3>Direct tests:</h3>
          ${rows.join("")}
        </div>
      </div>`;

      newInnerContainer.insertAdjacentHTML("beforeend", testsHTML);
    }

    if (typeof suite.s !== "undefined" && suite.s.length !== 0) {
      for (const innerSuite of suite.s) {
        addSuite(
          innerSuite,
          newInnerID,
          namespace + "/" + innerSuite.n,
          upstream
        );
      }
    }
  });

  newContainer.addEventListener("hidden.bs.collapse", (event) => {
    event.stopPropagation();
    newInnerContainer.innerHTML = "";
  });
}

function compareVersions(a, b) {
  a = splitVersion(a);
  b = splitVersion(b);
  if (a[0] > b[0]) {
    return 1;
  } else if (b[0] > a[0]) {
    return -1;
  } else if (a[1] > b[1]) {
    return 1;
  } else if (b[1] > a[1]) {
    return -1;
  } else if (a[2] > b[2]) {
    return 1;
  } else if (b[2] > a[2]) {
    return -1;
  } else {
    return 0;
  }
}

function splitVersion(version) {
  version = version[0] === "v" ? version.slice(1) : version;
  version = version.split(".").map((x) => parseInt(x));
  if (version.length === 1) {
    version.push(0);
  }
  if (version.length === 2) {
    version.push(0);
  }
  return version;
}
