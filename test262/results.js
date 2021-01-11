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
      let innerContainer = $("<div></div>")
        .addClass("card-body")
        .append($("<h2><code>master</code> branch results:</h2>"))
        .append(createGeneralInfo(data));

      if (typeof latest.master !== "undefined") {
        innerContainer.append(infoLink("master"));
      }

      $("#master-latest")
        .append($("<div></div>").addClass("card").append(innerContainer))
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
          let innerContainer = $("<div></div>")
            .addClass("card-body")
            .append($(`<h2>Latest version (${latestTag}) results:</h2>`))
            .append(createGeneralInfo(data));

          if (typeof latest[latestTag] !== "undefined") {
            innerContainer.append(infoLink(latestTag, data));
          }

          $("#version-latest")
            .append($("<div></div>").addClass("card").append(innerContainer))
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
    let container = $("<div></div>").addClass("info-link");

    if (tag === "master") {
      container.append(createHistoricalGraph());
    }

    container.append(
      $("<a></a>") // Bootstrap info-square icon:https://icons.getbootstrap.com/icons/info-square/
        .append($("<i></i>").addClass("bi").addClass("bi-info-square"))
        .addClass("card-link")
        .attr("href", "#")
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
        infoContainer.html("");
        let totalTests = data.r.c;
        let passedTests = data.r.o;
        let ignoredTests = data.r.i;
        let failedTests = totalTests - passedTests - ignoredTests;

        infoContainer.append(
          $("<div></div>")
            .addClass("card")
            .append(
              $("<div></div>")
                .addClass("progress")
                .addClass("progress-bar-striped")
                .append(
                  $("<div></div>")
                    .addClass("progress-bar")
                    .addClass("bg-success")
                    .attr("aria-valuenow", passedTests)
                    .attr("aria-valuemax", totalTests)
                    .attr("aria-valuemin", 0)
                    .css(
                      "width",
                      `${Math.round((passedTests / totalTests) * 10000) / 100}%`
                    )
                )
                .append(
                  $("<div></div>")
                    .addClass("progress-bar")
                    .addClass("bg-warning")
                    .attr("aria-valuenow", ignoredTests)
                    .attr("aria-valuemax", totalTests)
                    .attr("aria-valuemin", 0)
                    .css(
                      "width",
                      `${
                        Math.round((ignoredTests / totalTests) * 10000) / 100
                      }%`
                    )
                )
                .append(
                  $("<div></div>")
                    .addClass("progress-bar")
                    .addClass("bg-danger")
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
      let li = $("<div></div>").addClass("card");

      let newID = parentID + suite.n;
      let headerID = newID + "header";
      let header = $("<div></div>")
        .attr("id", headerID)
        .addClass("card-header")
        .addClass("col-md-12");

      // Add overal information:
      let info = $("<button></button>")
        .addClass("btn")
        .addClass("btn-light")
        .addClass("btn-block")
        .addClass("text-left")
        .attr("type", "button")
        .attr("data-toggle", "collapse");

      let name = $("<span></span>").addClass("name").text(suite.n);
      info.append(name).attr("aria-expanded", false);

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
      info.append($("<span></span>").addClass("data-overview").html(dataHTML));

      header.append(info);
      li.append(header);

      // Add sub-suites
      let inner = $("<div></div>")
        .attr("id", newID)
        .attr("data-parent", "#" + parentID)
        .addClass("collapse")
        .attr("aria-labelledby", headerID);

      let innerInner = $("<div></div>")
        .addClass("card-body")
        .addClass("accordion");

      if (typeof suite.t !== "undefined" && suite.t.length !== 0) {
        let grid = $("<div></div>")
          .addClass("card-body")
          .append($("<h3>Direct tests:</h3>"));
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

          let testCard = $("<div></div>")
            .attr("title", innerTest.n)
            .addClass("card")
            .addClass("test")
            .addClass("embed-responsive")
            .addClass(style)
            .click(() => {
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

        innerInner.append($("<div></div>").addClass("card").append(grid));
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
    return $("<ul></ul>")
      .addClass("list-group")
      .addClass("list-group-flush")
      .append(
        $("<li></li>")
          .addClass("list-group-item")
          .html(
            `Latest commit: <a href="https://github.com/boa-dev/boa/commit/${latest.c}" title="Check commit">${latest.c}</a>`
          )
      )
      .append(
        $("<li></li>")
          .addClass("list-group-item")
          .html(
            `Total tests: <span class="total-tests">${formatter.format(
              latest.t
            )}</span>`
          )
      )
      .append(
        $("<li></li>")
          .addClass("list-group-item")
          .html(
            `Passed tests: <span class="passed-tests">${formatter.format(
              latest.o
            )}</span>`
          )
      )
      .append(
        $("<li></li>")
          .addClass("list-group-item")
          .html(
            `Ignored tests: <span class="ignored-tests">${formatter.format(
              latest.i
            )}</span>`
          )
      )
      .append(
        $("<li></li>")
          .addClass("list-group-item")
          .html(
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
        $("<li></li>")
          .addClass("list-group-item")
          .html(
            `Conformance: <b>${
              Math.round((10000 * latest.o) / latest.t) / 100
            }%</b>`
          )
      );
  }

  function createHistoricalGraph() {
    let options = {
      content: graph,
      html: true,
      placement: "bottom",
      container: "body",
    };

    return $("<a></a>")
      .append(
        $("<i></i>")
          .addClass("bi")
          .addClass("bi-graph-up")
          .on('shown.bs.modal', () => {
            let graph = $("#master-graph");

            new Chart(graph, {
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
                    data: masterData.map((data) => data.i + data.o),
                    backgroundColor: "#dfa800",
                    borderColor: "#6f5400",
                    borderWidth: 1,
                  },
                  {
                    label: "Panics",
                    data: masterData.map((data) => data.i + data.o + data.p),
                    backgroundColor: "#a30000",
                    borderColor: "#510000",
                    borderWidth: 1,
                  },
                  {
                    label: "Failed",
                    data: masterData.map((data) => data.t),
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
                  display: false,
                },
                responsive: false,
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
                      display: true,
                    },
                  ],
                },
              },
            });
          })
          .modal(options)
      )
      .addClass("card-link")
      .attr("href", "#")
      .click(() => {
        $("#graph-modal").modal('show');
      });
  }

  function getRefTag(tag) {
    let version = tag.split(".");

    // Seems that refs are stored with an ending 0:
    if (version.length == 2) {
      tag += ".0";
    }

    return [version, tag];
  }
})();
