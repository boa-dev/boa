"use strict";

(function () {
  let latest = {};
  let mainData = [];
  let formatter = new Intl.NumberFormat("en-GB");

  // Load latest complete data from main:
  fetch("./refs/heads/main/latest.json")
    .then((response) => response.json())
    .then((data) => {
      latest.main = data;

      let container = $("#main-latest .card-body");
      container.append(infoLink("main"));
    });

  // Load main branch information over time:
  fetch("./refs/heads/main/results.json")
    .then((response) => response.json())
    .then((data) => {
      mainData = data;
      let innerContainer = $('<div class="card-body"></div>')
        .append($("<h2><code>main</code> branch results:</h2>"))
        .append(createGeneralInfo(data));

      if (typeof latest.main !== "undefined") {
        innerContainer.append(infoLink("main"));
      }

      $("#main-latest")
        .append($('<div class="card"></div>').append(innerContainer))
        .show();
    });

  // Tags/releases information.
  fetch("https://api.github.com/repos/boa-dev/boa/releases")
    .then((response) => response.json())
    .then((data) => {
      data.sort((a, b) => compareVersions(a.tag_name, b.tag_name) * -1);
      let latestTag = data[0].tag_name;

      // We set the latest version.
      fetch(`./refs/tags/${getRefTag(latestTag)[1]}/results.json`)
        .then((response) => response.json())
        .then((data) => {
          let innerContainer = $('<div class="card-body"></div>')
            .append($(`<h2>Latest version (${latestTag}) results:</h2>`))
            .append(createGeneralInfo(data));

          if (typeof latest[latestTag] !== "undefined") {
            innerContainer.append(infoLink(latestTag));
          }

          $("#version-latest")
            .append($('<div class="card"></div>').append(innerContainer))
            .show();
        });

      let versionList = [];

      for (let rel of data) {
        let [version, tag] = getRefTag(rel.tag_name);

        if (version[0] == "v0" && parseInt(version[1]) < 10) {
          // We know there is no data for versions lower than v0.10.
          continue;
        }

        fetch(`./refs/tags/${tag}/latest.json`)
          .then((response) => response.json())
          .then((reldata) => {
            latest[rel.tag_name] = reldata;

            if (rel.tag_name == latestTag) {
              let container = $("#version-latest .card-body");
              container.append(infoLink(rel.tag_name));
              return;
            }

            let dataHTML =
              '<span class="position-absolute top-50 start-50 translate-middle">';
            dataHTML += `<span class="text-success">${formatter.format(
              reldata.r.o
            )}</span>`;
            dataHTML += ` / <span class="text-warning">${formatter.format(
              reldata.r.i
            )}</span>`;
            dataHTML += ` / <span class="text-danger">${formatter.format(
              reldata.r.c - reldata.r.o - reldata.r.i
            )}${
              reldata.r.p !== 0
                ? ` (${formatter.format(
                    reldata.r.p
                  )} <i class="bi-exclamation-triangle"></i>)`
                : ""
            }</span>`;
            console.log(reldata.r);
            dataHTML += ` / <b>${formatter.format(
              Math.round((10000 * reldata.r.o) / reldata.r.c) / 100
            )}%</b></span>`;

            var tagHTML = `<b class="position-absolute top-50 start-0 translate-middle-y">${tag}</b>`;

            let html = $(
              `<li class="list-group-item position-relative">${tagHTML}${dataHTML}</li>`
            );
            html.append(
              infoLink(
                rel.tag_name,
                "position-absolute top-50 end-0 translate-middle-y"
              )
            );
            versionList.push({ tag, html });
            //   .append(createGeneralInfo(data));

            // if (typeof latest[latestTag] !== "undefined") {
            //   innerContainer.append();
            // }

            if (versionList.length === data.length - 11) {
              versionList.sort((a, b) => compareVersions(a.tag, b.tag) * -1);

              let versionListHTML = $(
                '<ul class="list-group list-group-flush"></ul>'
              );
              for (version of versionList) {
                versionListHTML.append(version.html);
              }

              $("#old-versions")
                .append(
                  $('<div class="card"></div>').append(
                    $('<div class="card-body"></div>')
                      .append($(`<h2>Older versions</h2>`))
                      .append(versionListHTML)
                  )
                )
                .show();
            }
          });
      }
    });

  // Creates a link to show the information about a particular tag / branch
  function infoLink(tag, extraClass) {
    let container = $(
      `<div class="info-link${extraClass ? " " + extraClass : ""}"></div>`
    );

    if (tag === "main") {
      container.append(createHistoricalGraph());
    }

    container.append(
      $('<a class="card-link"></a>')
        .append($('<i class="bi-info-square"></i>'))
        .on("click", (e) => {
          let data = latest[tag];
          showData(data, e.target);
        })
    );

    return container;
  }

  // Shows the full test data.
  function showData(data, infoIcon) {
    let infoContainer = $("#infoContainer");
    let info = $("#info");
    $(infoIcon).attr("class", "spinner-border text-primary small");

    setTimeout(
      function () {
        info.empty();
        let totalTests = data.r.c;
        let passedTests = data.r.o;
        let ignoredTests = data.r.i;
        let failedTests = totalTests - passedTests - ignoredTests;

        infoContainer.prepend(
          $('<div class="progress g-0"></div>')
            .append(
              $(
                '<div class="progress-bar progress-bar-striped bg-success"></div>'
              )
                .attr("aria-valuenow", passedTests)
                .attr("aria-valuemax", totalTests)
                .attr("aria-valuemin", 0)
                .attr("role", "progressbar")
                .css(
                  "width",
                  `${Math.round((passedTests / totalTests) * 10000) / 100}%`
                )
            )
            .append(
              $(
                '<div class="progress-bar progress-bar-striped bg-warning"></div>'
              )
                .attr("aria-valuenow", ignoredTests)
                .attr("aria-valuemax", totalTests)
                .attr("aria-valuemin", 0)
                .attr("role", "progressbar")
                .css(
                  "width",
                  `${Math.round((ignoredTests / totalTests) * 10000) / 100}%`
                )
            )
            .append(
              $(
                '<div class="progress-bar progress-bar-striped bg-danger"></div>'
              )
                .attr("aria-valuenow", failedTests)
                .attr("aria-valuemax", totalTests)
                .attr("aria-valuemin", 0)
                .attr("role", "progressbar")
                .css(
                  "width",
                  `${Math.round((failedTests / totalTests) * 10000) / 100}%`
                )
            )
        );

        for (let suite of data.r.s) {
          addSuite(info, suite, "info", "test/" + suite.n, data.u);
        }
        infoContainer.collapse("show");
        $(infoIcon).attr("class", "bi-info-square");
      },
      infoContainer.hasClass("show") ? 500 : 0
    );
    infoContainer.collapse("hide");

    // Adds a suite representation to an element.
    function addSuite(elm, suite, parentID, namespace, upstream) {
      let li = $('<div class="card g-0"></div>');

      let newID = parentID + suite.n;
      let headerID = newID + "header";
      let header = $(
        `<div id="${headerID}" class="card-header col-md-12 d-grid"></div>`
      );

      // Add overal information:
      let info = $(
        '<button type="button" aria-expanded="false" data-bs-toggle="collapse" class="btn btn-light text-start"></button>'
      );

      let name = $('<span class="name"></span>').text(suite.n);
      info.append(name);

      let dataHTML = ` <span class="text-success">${formatter.format(
        suite.o
      )}</span>`;
      dataHTML += ` / <span class="text-warning">${formatter.format(
        suite.i
      )}</span>`;
      dataHTML += ` / <span class="text-danger">${formatter.format(
        suite.c - suite.o - suite.i
      )}${
        suite.p !== 0
          ? ` (${formatter.format(
              suite.p
            )} <i class="bi-exclamation-triangle"></i>)`
          : ""
      }</span>`;
      dataHTML += ` / <span>${formatter.format(suite.c)}</span>`;
      info.append($('<span class="data-overview"></span>').html(dataHTML));

      header.append(info);
      li.append(header);

      // Add sub-suites
      let inner = $(
        `<div id="${newID}" data-bs-parent="#${parentID}" class="collapse" aria-labelledby="${headerID}"></div>`
      );

      let innerInner = $('<div class="card-body accordion"></div>');

      if (typeof suite.t !== "undefined" && suite.t.length !== 0) {
        let grid = $('<div class="row card-body"></div>').append(
          $("<h3>Direct tests:</h3>")
        );
        for (let innerTest of suite.t) {
          let name = namespace + "/" + innerTest.n + ".js";
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

          let testCard = $(
            `<a title="${innerTest.n}" class="card test embed-responsive ${style}"></a>`
          )
            .attr(
              "href",
              `https://github.com/tc39/test262/blob/${upstream}/${name}`
            )
            .attr("target", "_blank");

          if (innerTest.r === "P") {
            testCard.append($('<i class="bi-exclamation-triangle"></i>'));
          } else {
            testCard.addClass("embed-responsive-1by1");
          }

          grid.append(testCard);
        }

        innerInner.append($('<div class="card"></div>').append(grid));
      }

      if (typeof suite.s !== "undefined" && suite.s.length !== 0) {
        for (let innerSuite of suite.s) {
          addSuite(
            innerInner,
            innerSuite,
            newID,
            namespace + "/" + innerSuite.n,
            upstream
          );
        }
      }

      info.attr("aria-controls", newID).attr("data-bs-target", "#" + newID);
      inner.on('show.bs.collapse', {elem: innerInner}, function(event){
        event.data.elem.appendTo(inner);
      });
      inner.on('hidden.bs.collapse', {elem: innerInner}, function(event){
        event.stopPropagation();
        event.data.elem.detach();
      });
      li.append(inner);

      elm.append(li);
    }
  }

  /// Creates the general information structure.
  function createGeneralInfo(data) {
    let latest = data[data.length - 1];
    return $('<ul class="list-group list-group-flush"></ul>')
      .append(
        $('<li class="list-group-item"></li>').html(
          `Latest commit: <a href="https://github.com/boa-dev/boa/commit/${latest.c}" title="Check commit">${latest.c}</a>`
        )
      )
      .append(
        $('<li class="list-group-item"></li>').html(
          `Total tests: <span>${formatter.format(latest.t)}</span>`
        )
      )
      .append(
        $('<li class="list-group-item"></li>').html(
          `Passed tests: <span class="text-success">${formatter.format(
            latest.o
          )}</span>`
        )
      )
      .append(
        $('<li class="list-group-item"></li>').html(
          `Ignored tests: <span class="text-warning">${formatter.format(
            latest.i
          )}</span>`
        )
      )
      .append(
        $('<li class="list-group-item"></li>').html(
          `Failed tests: <span class="text-danger">${formatter.format(
            latest.t - latest.o - latest.i
          )}${
            latest.p !== 0
              ? ` (${formatter.format(
                  latest.p
                )} <i class="bi-exclamation-triangle"></i>)`
              : ""
          }</span>`
        )
      )
      .append(
        $('<li class="list-group-item"></li>').html(
          `Conformance: <b>${
            Math.round((10000 * latest.o) / latest.t) / 100
          }%</b>`
        )
      );
  }

  function createHistoricalGraph() {
    $("#graph-modal .modal-body").append(
      $('<canvas id="main-graph"></canvas>')
    );

    $("#graph-modal").on("hidden.bs.modal", () => {
      $("#graph-modal .modal-body").empty();
      $("#graph-modal .modal-body").append(
        $('<canvas id="main-graph"></canvas>')
      );
    });

    $("#graph-modal").on("shown.bs.modal", () => {
      new Chart($("#main-graph"), {
        type: "line",
        data: {
          labels: mainData.map((data) => data.c),
          datasets: [
            {
              label: "Passed",
              data: mainData.map((data) => data.o),
              backgroundColor: "#1fcb4a",
              borderColor: "#0f6524",
              borderWidth: 1,
              fill: true,
            },
            {
              label: "Ignored",
              data: mainData.map((data) => data.i),
              backgroundColor: "#dfa800",
              borderColor: "#6f5400",
              borderWidth: 1,
              fill: true,
            },
            {
              label: "Panics",
              data: mainData.map((data) => data.p),
              backgroundColor: "#a30000",
              borderColor: "#510000",
              borderWidth: 1,
              fill: true,
            },
            {
              label: "Failed",
              data: mainData.map((data) => data.t - data.i - data.o - data.p),
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
    });

    return $('<a class="card-link" href="#""></a>')
      .append($('<i class="bi-graph-up"></i>'))
      .click(() => {
        $("#graph-modal").modal("show");
      });
  }

  function getRefTag(tag) {
    let version = tag.split(".");

    return [version, tag];
  }
})();

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

function splitVersion(ver) {
  ver = ver[0] === "v" ? ver.slice(1) : ver;
  ver = ver.split(".").map((x) => parseInt(x));

  if (ver.length === 1) {
    ver.push(0);
  }

  if (ver.length === 2) {
    ver.push(0);
  }

  return ver;
}
