"use strict";
(function () {
  let latest = {};
  let masterData = [];
  let formatter = new Intl.NumberFormat("en-GB");

  // Load latest complete data from master:
  fetch("./refs/heads/master/latest.json")
    .then((response) => response.json())
    .then((data) => {
      latest.master = data;

      let container = $("#master-latest .card-body");
      container.append(infoLink("master"));
    });

  // Load master branch information over time:
  fetch("./refs/heads/master/results.json")
    .then((response) => response.json())
    .then((data) => {
      masterData = data;
      let innerContainer = $('<div class="card-body"></div>')
        .append($("<h2><code>master</code> branch results:</h2>"))
        .append(createGeneralInfo(data));

      if (typeof latest.master !== "undefined") {
        innerContainer.append(infoLink("master"));
      }

      $("#master-latest")
        .append($('<div class="card"></div>').append(innerContainer))
        .show();
    });

  // Tags/releases information.
  fetch("https://api.github.com/repos/boa-dev/boa/releases")
    .then((response) => response.json())
    .then((data) => {
      let latestTag = data[0].tag_name;

      // We set the latest version.
      fetch(`./refs/tags/${getRefTag(latestTag)[1]}/results.json`)
        .then((response) => response.json())
        .then((data) => {
          let innerContainer = $('<div class="card-body"></div>')
            .append($(`<h2>Latest version (${latestTag}) results:</h2>`))
            .append(createGeneralInfo(data));

          if (typeof latest[latestTag] !== "undefined") {
            innerContainer.append(infoLink(latestTag, data));
          }

          $("#version-latest")
            .append($('<div class="card"></div>').append(innerContainer))
            .show();
        });

      for (let rel of data) {
        let [version, tag] = getRefTag(rel.tag_name);

        if (version[0] == "v0" && parseInt(version[1]) < 10) {
          // We know there is no data for versions lower than v0.10.
          continue;
        }

        fetch(`./refs/tags/${tag}/latest.json`)
          .then((response) => response.json())
          .then((data) => {
            latest[rel.tag_name] = data;

            if (rel.tag_name == latestTag) {
              let container = $("#version-latest .card-body");
              container.append(infoLink(rel.tag_name));
            }

            // TODO: add version history.
          });
      }
    });

  // Creates a link to show the information about a particular tag / branch
  function infoLink(tag, data) {
    let container = $('<div class="info-link"></div>');

    if (tag === "master") {
      container.append(createHistoricalGraph());
    }

    container.append(
      $('<a class="card-link" href="#"></a>')
        .append($('<i class="bi bi-info-square"></i>'))
        .click(() => {
          let data = latest[tag];
          showData(data);
        })
    );

    return container;
  }

  // Shows the full test data.
  function showData(data) {
    let infoContainer = $("#info");
    setTimeout(
      function () {
        infoContainer.empty();
        let totalTests = data.r.c;
        let passedTests = data.r.o;
        let ignoredTests = data.r.i;
        let failedTests = totalTests - passedTests - ignoredTests;

        infoContainer.append(
          $('<div class="card"></div>').append(
            $('<div class="progress progress-bar-stripped"></div>')
              .append(
                $('<div class="progress-bar bg-success"></div>')
                  .attr("aria-valuenow", passedTests)
                  .attr("aria-valuemax", totalTests)
                  .attr("aria-valuemin", 0)
                  .css(
                    "width",
                    `${Math.round((passedTests / totalTests) * 10000) / 100}%`
                  )
              )
              .append(
                $('<div class="progress-bar bg-warning"></div>')
                  .attr("aria-valuenow", ignoredTests)
                  .attr("aria-valuemax", totalTests)
                  .attr("aria-valuemin", 0)
                  .css(
                    "width",
                    `${Math.round((ignoredTests / totalTests) * 10000) / 100}%`
                  )
              )
              .append(
                $('<div class="progress-bar bg-danger"></div>')
                  .attr("aria-valuenow", failedTests)
                  .attr("aria-valuemax", totalTests)
                  .attr("aria-valuemin", 0)
                  .css(
                    "width",
                    `${Math.round((failedTests / totalTests) * 10000) / 100}%`
                  )
              )
          )
        );

        for (let suite of data.r.s) {
          addSuite(infoContainer, suite, "info", "test/" + suite.n, data.u);
        }
        infoContainer.collapse("show");
      },
      infoContainer.hasClass("show") ? 500 : 0
    );
    infoContainer.collapse("hide");

    // Adds a suite representation to an element.
    function addSuite(elm, suite, parentID, namespace, upstream) {
      let li = $('<div class="card"></div>');

      let newID = parentID + suite.n;
      let headerID = newID + "header";
      let header = $(
        `<div id="${headerID}" class="card-header col-md-12"></div>`
      );

      // Add overal information:
      let info = $(
        '<button type="button" aria-expanded="false" data-toggle="collapse" class="btn btn-light btn-block text-left"></button>'
      );

      let name = $('<span class="name"></span>').text(suite.n);
      info.append(name);

      let dataHTML = ` <span class="passed-tests">${formatter.format(
        suite.o
      )}</span>`;
      dataHTML += ` / <span class="ignored-tests">${formatter.format(
        suite.i
      )}</span>`;
      dataHTML += ` / <span class="failed-tests">${formatter.format(
        suite.c - suite.o - suite.i
      )}${
        suite.p !== 0
          ? ` (${formatter.format(
              suite.p
            )} <i class="bi bi-exclamation-triangle"></i>)`
          : ""
      }</span>`;
      dataHTML += ` / <span class="total-tests">${formatter.format(
        suite.c
      )}</span>`;
      info.append($('<span class="data-overview"></span>').html(dataHTML));

      header.append(info);
      li.append(header);

      // Add sub-suites
      let inner = $(
        `<div id="${newID}" data-parent="#${parentID}" class="collapse" aria-labelledby="${headerID}"></div>`
      );

      let innerInner = $('<div class="card-body accordion"></div>');

      if (typeof suite.t !== "undefined" && suite.t.length !== 0) {
        let grid = $('<div class="card-body"></div>').append(
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
            `<div title="${innerTest.n}" class="card test embed-responsive ${style}"></div>`
          ).click(() => {
            window.open(
              `https://github.com/tc39/test262/blob/${upstream}/${name}`
            );
          });

          if (innerTest.r === "P") {
            testCard.append(
              $("<i></i>").addClass("bi").addClass("bi-exclamation-triangle")
            );
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

      info.attr("aria-controls", newID).attr("data-target", "#" + newID);
      inner.append(innerInner);
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
          `Total tests: <span class="total-tests">${formatter.format(
            latest.t
          )}</span>`
        )
      )
      .append(
        $('<li class="list-group-item"></li>').html(
          `Passed tests: <span class="passed-tests">${formatter.format(
            latest.o
          )}</span>`
        )
      )
      .append(
        $('<li class="list-group-item"></li>').html(
          `Ignored tests: <span class="ignored-tests">${formatter.format(
            latest.i
          )}</span>`
        )
      )
      .append(
        $('<li class="list-group-item"></li>').html(
          `Failed tests: <span class="failed-tests">${formatter.format(
            latest.t - latest.o - latest.i
          )}${
            latest.p !== 0
              ? ` (${formatter.format(
                  latest.p
                )} <i class="bi bi-exclamation-triangle"></i>)`
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
      $('<canvas id="master-graph"></canvas>')
    );

    $("#graph-modal").on("hidden.bs.modal", () => {
      $("#graph-modal .modal-body").empty();
      $("#graph-modal .modal-body").append(
        $('<canvas id="master-graph"></canvas>')
      );
    });

    $("#graph-modal").on("shown.bs.modal", () => {
      new Chart($("#master-graph"), {
        type: "line",
        data: {
          labels: masterData.map((data) => data.c),
          datasets: [
            {
              label: "Passed",
              data: masterData.map((data) => data.o),
              backgroundColor: "#1fcb4a",
              borderColor: "#0f6524",
              borderWidth: 1,
            },
            {
              label: "Ignored",
              data: masterData.map((data) => data.i),
              backgroundColor: "#dfa800",
              borderColor: "#6f5400",
              borderWidth: 1,
            },
            {
              label: "Panics",
              data: masterData.map((data) => data.p),
              backgroundColor: "#a30000",
              borderColor: "#510000",
              borderWidth: 1,
            },
            {
              label: "Failed",
              data: masterData.map((data) => data.t - data.i - data.o - data.p),
              backgroundColor: "#ff4848",
              borderColor: "#a30000",
              borderWidth: 1,
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
          hover: {
            mode: "nearest",
          },
          scales: {
            xAxes: [
              {
                display: false,
              },
            ],
            yAxes: [
              {
                stacked: true,
                scaleLabel: {
                  display: true,
                  labelString: "Tests",
                },
              },
            ],
          },
        },
      });
    });

    return $('<a class="card-link" href="#""></a>')
      .append($('<i class="bi bi-graph-up"></i>'))
      .click(() => {
        $("#graph-modal").modal("show");
      });
  }

  function getRefTag(tag) {
    let version = tag.split(".");

    return [version, tag];
  }
})();
