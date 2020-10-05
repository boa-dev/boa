"use strict";
(function () {
  let latest = {};
  let formatter = new Intl.NumberFormat("en-GB");

  // Load latest complete data from master:
  fetch("/test262/refs/heads/master/latest.json")
    .then((response) => response.json())
    .then((data) => {
      latest.master = data;

      let container = $("#master-latest .card-body");
      container.append(infoLink("master"));
    });

  // Load master branch information over time:
  fetch("/test262/refs/heads/master/results.json")
    .then((response) => response.json())
    .then((data) => {
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

      // TODO: paint the graph with historical data.
    });

  // Tags/releases information.
  fetch("https://api.github.com/repos/boa-dev/boa/releases")
    .then((response) => response.json())
    .then((data) => {
      let latestTag = data[0].tag_name;

      // We set the latest version.
      fetch(`/test262/refs/tags/${getRefTag(latestTag)[1]}/results.json`)
        .then((response) => response.json())
        .then((data) => {
          let innerContainer = $("<div></div>")
            .addClass("card-body")
            .append($(`<h2>Latest version (${latestTag}) results:</h2>`))
            .append(createGeneralInfo(data));

          if (typeof latest[latestTag] !== "undefined") {
            innerContainer.append(infoLink(latestTag));
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

        fetch(`/test262/refs/tags/${tag}/latest.json`)
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
  function infoLink(tag) {
    return $("<div></div>")
      .addClass("info-link")
      .append(
        $("<a></a>") // Bootstrap info-square icon:https://icons.getbootstrap.com/icons/info-square/
          .append($("<span></span>").addClass("info-link"))
          .addClass("card-link")
          .attr("href", "#")
          .click(() => {
            let data = latest[tag];
            showData(data);
          })
      );
  }

  // Shows the full test data.
  function showData(data) {
    let infoContainer = $("#info");
    setTimeout(
      function () {
        infoContainer.html("");
        let totalTests = data.r.c;
        let passedTests = data.r.p;
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
        suite.p
      )}</span>`;
      dataHTML += ` / <span class="ignored-tests">${formatter.format(
        suite.i
      )}</span>`;
      dataHTML += ` / <span class="failed-tests">${formatter.format(
        suite.c - suite.p - suite.i
      )}</span>`;
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
              $("<span></span>").addClass("exclamation-triangle")
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

  // Displays test information in a modal.
  function displayTestModal(name) {
    fetch("https://raw.githubusercontent.com/tc39/test262/main/" + name)
      .then((response) => response.text())
      .then((code) => console.log(code));
    // console.log(test262Info[name]);
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
              latest.p
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
              latest.t - latest.p - latest.i
            )}</span>`
          )
      )
      .append(
        $("<li></li>")
          .addClass("list-group-item")
          .html(
            `Conformance: <b>${
              Math.round((10000 * latest.p) / latest.t) / 100
            }%</b>`
          )
        // TODO: add progress bar
      );
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
