import init, { RhuideanStudio } from "../pkg/rhuidean_studio.js";

async function main() {
    await init();

    const canvas = document.getElementById("canvas");
    const dpr = window.devicePixelRatio || 1;

    function sizeCanvas() {
        const rect = canvas.getBoundingClientRect();
        canvas.width = rect.width * dpr;
        canvas.height = rect.height * dpr;
        return { w: canvas.width, h: canvas.height };
    }

    let { w, h } = sizeCanvas();
    const app = new RhuideanStudio(canvas);
    app.resize(w, h);

    window.addEventListener("resize", () => {
        let { w, h } = sizeCanvas();
        app.resize(w, h);
        if (bloomActive) {
            sizeBloomCanvas();
        }
        ensureLoop();
    });

    const intervals = app.get_intervals_json();
    buildRatioSelect(intervals);

    const ratioEl = document.getElementById("ratio");
    const customGroup = document.getElementById("custom-ratio-group");
    const customP = document.getElementById("custom-p");
    const customQ = document.getElementById("custom-q");
    const customName = document.getElementById("custom-name");
    const orbitsEl = document.getElementById("orbits");
    const orbitsVal = document.getElementById("orbits-val");
    const velocityEl = document.getElementById("velocity-mode");
    const tuningEl = document.getElementById("tuning");
    const waveformEl = document.getElementById("waveform");
    const subdivisionsEl = document.getElementById("subdivisions");
    const subdivisionsVal = document.getElementById("subdivisions-val");
    const speedEl = document.getElementById("speed");
    const speedVal = document.getElementById("speed-val");
    const baseFreqEl = document.getElementById("base-freq");
    const freqVal = document.getElementById("freq-val");
    const btnStart = document.getElementById("btn-start");
    const btnStop = document.getElementById("btn-stop");
    const btnReset = document.getElementById("btn-reset");

    ratioEl.addEventListener("change", () => {
        const v = ratioEl.value;
        if (v === "custom") {
            customGroup.style.display = "";
            applyCustomRatio();
        } else {
            customGroup.style.display = "none";
            const [p, q] = v.split("/").map(Number);
            app.set_ratio(p, q);
        }
        ensureLoop();
    });

    function applyCustomRatio() {
        const p = parseInt(customP.value) || 1;
        const q = parseInt(customQ.value) || 1;
        app.set_ratio(p, q);
        const match = intervals.find((i) => i.numerator === p && i.denominator === q);
        customName.textContent = match ? match.name : "";
    }

    customP.addEventListener("input", () => { applyCustomRatio(); ensureLoop(); });
    customQ.addEventListener("input", () => { applyCustomRatio(); ensureLoop(); });

    orbitsEl.addEventListener("input", () => {
        const n = parseInt(orbitsEl.value);
        orbitsVal.textContent = n;
        app.set_num_orbits(n);
        ensureLoop();
    });

    velocityEl.addEventListener("change", () => { app.set_velocity_mode(velocityEl.value); ensureLoop(); });
    tuningEl.addEventListener("change", () => { app.set_tuning(tuningEl.value); ensureLoop(); });
    waveformEl.addEventListener("change", () => { app.set_waveform(waveformEl.value); ensureLoop(); });

    subdivisionsEl.addEventListener("input", () => {
        const n = parseInt(subdivisionsEl.value);
        subdivisionsVal.textContent = n;
        app.set_subdivisions(n);
        ensureLoop();
    });

    speedEl.addEventListener("input", () => {
        const s = parseFloat(speedEl.value);
        speedVal.textContent = s.toFixed(1);
        app.set_speed(s);
    });

    baseFreqEl.addEventListener("input", () => {
        const f = parseInt(baseFreqEl.value);
        freqVal.textContent = f;
        app.set_base_freq(f);
    });

    const filterEnabled = document.getElementById("filter-enabled");
    const filterCutoff = document.getElementById("filter-cutoff");
    const cutoffVal = document.getElementById("cutoff-val");
    const filterResonance = document.getElementById("filter-resonance");
    const resonanceVal = document.getElementById("resonance-val");
    const delayWet = document.getElementById("delay-wet");
    const delayWetVal = document.getElementById("delay-wet-val");
    const delayTime = document.getElementById("delay-time");
    const delayTimeVal = document.getElementById("delay-time-val");
    const delayFeedback = document.getElementById("delay-feedback");
    const delayFbVal = document.getElementById("delay-fb-val");
    const stereoEnabled = document.getElementById("stereo-enabled");
    const detuneEl = document.getElementById("detune");
    const detuneVal = document.getElementById("detune-val");
    const chordEnabled = document.getElementById("chord-enabled");
    const convergenceLines = document.getElementById("convergence-lines");
    const spiralTrails = document.getElementById("spiral-trails");
    const themeEl = document.getElementById("theme");

    filterEnabled.addEventListener("change", () => app.set_filter_enabled(filterEnabled.checked));

    filterCutoff.addEventListener("input", () => {
        const v = parseInt(filterCutoff.value);
        cutoffVal.textContent = v;
        app.set_filter_cutoff(v);
    });

    filterResonance.addEventListener("input", () => {
        const v = parseFloat(filterResonance.value);
        resonanceVal.textContent = v.toFixed(1);
        app.set_filter_resonance(v);
    });

    delayWet.addEventListener("input", () => {
        const v = parseFloat(delayWet.value);
        delayWetVal.textContent = Math.round(v * 100) + "%";
        app.set_delay_wet(v);
    });

    delayTime.addEventListener("input", () => {
        const v = parseFloat(delayTime.value);
        delayTimeVal.textContent = v.toFixed(2);
        app.set_delay_time(v);
    });

    delayFeedback.addEventListener("input", () => {
        const v = parseFloat(delayFeedback.value);
        delayFbVal.textContent = Math.round(v * 100) + "%";
        app.set_delay_feedback(v);
    });

    stereoEnabled.addEventListener("change", () => app.set_stereo_enabled(stereoEnabled.checked));

    detuneEl.addEventListener("input", () => {
        const v = parseInt(detuneEl.value);
        detuneVal.textContent = v;
        app.set_detune_amount(v);
    });

    chordEnabled.addEventListener("change", () => app.set_chord_enabled(chordEnabled.checked));

    convergenceLines.addEventListener("change", () => {
        app.set_convergence_lines(convergenceLines.checked);
        ensureLoop();
    });

    spiralTrails.addEventListener("change", () => {
        app.set_spiral_trails(spiralTrails.checked);
        ensureLoop();
    });

    const spiralMode = document.getElementById("spiral-mode");
    const spiralBlend = document.getElementById("spiral-blend");
    const spiralBlendVal = document.getElementById("spiral-blend-val");

    spiralMode.addEventListener("change", () => {
        app.set_spiral_mode(spiralMode.value);
        app.set_spiral_trails(false);
        app.set_spiral_trails(spiralTrails.checked);
        ensureLoop();
    });

    spiralBlend.addEventListener("input", () => {
        const v = parseInt(spiralBlend.value);
        spiralBlendVal.textContent = v + "%";
        app.set_spiral_blend(v / 100);
    });

    themeEl.addEventListener("change", () => {
        app.set_theme(themeEl.value);
        ensureLoop();
    });

    const bloomEnabled = document.getElementById("bloom-enabled");
    const bloomIntensity = document.getElementById("bloom-intensity");
    const bloomVal = document.getElementById("bloom-val");
    const bloomCanvas = document.getElementById("bloom-canvas");
    const bloomCtx = bloomCanvas.getContext("2d");
    let bloomActive = false;

    function sizeBloomCanvas() {
        const rect = canvas.getBoundingClientRect();
        bloomCanvas.width = rect.width * dpr;
        bloomCanvas.height = rect.height * dpr;
        bloomCanvas.style.width = rect.width + "px";
        bloomCanvas.style.height = rect.height + "px";
    }

    bloomEnabled.addEventListener("change", () => {
        bloomActive = bloomEnabled.checked;
        bloomCanvas.style.display = bloomActive ? "block" : "none";
        if (bloomActive) {
            sizeBloomCanvas();
            bloomCanvas.style.opacity = parseInt(bloomIntensity.value) / 100;
        }
    });

    bloomIntensity.addEventListener("input", () => {
        const v = parseInt(bloomIntensity.value);
        bloomVal.textContent = v + "%";
        bloomCanvas.style.opacity = v / 100;
    });

    let animationId = null;

    function loop(timestamp) {
        app.frame(timestamp);
        if (bloomActive) {
            bloomCtx.clearRect(0, 0, bloomCanvas.width, bloomCanvas.height);
            bloomCtx.drawImage(canvas, 0, 0);
        }
        if (app.is_running()) {
            animationId = requestAnimationFrame(loop);
        } else {
            animationId = null;
        }
    }

    function ensureLoop() {
        if (animationId === null) {
            animationId = requestAnimationFrame(loop);
        }
    }

    btnStart.addEventListener("click", () => {
        app.start();
        btnStart.disabled = true;
        btnStop.disabled = false;
        ensureLoop();
    });

    btnStop.addEventListener("click", () => {
        app.stop();
        btnStart.disabled = false;
        btnStop.disabled = true;
    });

    btnReset.addEventListener("click", () => {
        app.reset();
        ensureLoop();
    });

    const btnDefaults = document.getElementById("btn-defaults");
    btnDefaults.addEventListener("click", () => {
        ratioEl.value = "3/2";
        customGroup.style.display = "none";
        customP.value = 3;
        customQ.value = 2;
        customName.textContent = "";
        orbitsEl.value = 12;
        velocityEl.value = "linear";
        tuningEl.value = "overtone";
        waveformEl.value = "sine";
        subdivisionsEl.value = 1;
        speedEl.value = 1;
        baseFreqEl.value = 220;
        filterEnabled.checked = false;
        filterCutoff.value = 4000;
        filterResonance.value = 2;
        delayWet.value = 0;
        delayTime.value = 0.3;
        delayFeedback.value = 0.4;
        stereoEnabled.checked = false;
        detuneEl.value = 0;
        chordEnabled.checked = false;
        convergenceLines.checked = false;
        spiralTrails.checked = false;
        spiralMode.value = "epitrochoid";
        spiralBlend.value = 50;
        themeEl.value = "default";
        bloomEnabled.checked = false;
        bloomIntensity.value = 50;
        bloomActive = false;
        bloomCanvas.style.display = "none";
        syncAll();
        app.reset();
        ensureLoop();
    });

    function syncAll() {
        const rv = ratioEl.value;
        if (rv === "custom") {
            customGroup.style.display = "";
            applyCustomRatio();
        } else if (rv) {
            const [p, q] = rv.split("/").map(Number);
            app.set_ratio(p, q);
        }

        const n = parseInt(orbitsEl.value);
        orbitsVal.textContent = n;
        app.set_num_orbits(n);

        app.set_velocity_mode(velocityEl.value);
        app.set_tuning(tuningEl.value);
        app.set_waveform(waveformEl.value);

        const sub = parseInt(subdivisionsEl.value);
        subdivisionsVal.textContent = sub;
        app.set_subdivisions(sub);

        const s = parseFloat(speedEl.value);
        speedVal.textContent = s.toFixed(1);
        app.set_speed(s);

        const f = parseInt(baseFreqEl.value);
        freqVal.textContent = f;
        app.set_base_freq(f);

        app.set_filter_enabled(filterEnabled.checked);
        cutoffVal.textContent = parseInt(filterCutoff.value);
        app.set_filter_cutoff(parseInt(filterCutoff.value));
        resonanceVal.textContent = parseFloat(filterResonance.value).toFixed(1);
        app.set_filter_resonance(parseFloat(filterResonance.value));

        const dw = parseFloat(delayWet.value);
        delayWetVal.textContent = Math.round(dw * 100) + "%";
        app.set_delay_wet(dw);
        const dt = parseFloat(delayTime.value);
        delayTimeVal.textContent = dt.toFixed(2);
        app.set_delay_time(dt);
        const df = parseFloat(delayFeedback.value);
        delayFbVal.textContent = Math.round(df * 100) + "%";
        app.set_delay_feedback(df);

        app.set_stereo_enabled(stereoEnabled.checked);

        const det = parseInt(detuneEl.value);
        detuneVal.textContent = det;
        app.set_detune_amount(det);

        app.set_chord_enabled(chordEnabled.checked);
        app.set_convergence_lines(convergenceLines.checked);
        app.set_spiral_trails(spiralTrails.checked);
        app.set_spiral_mode(spiralMode.value);
        const sb = parseInt(spiralBlend.value);
        spiralBlendVal.textContent = sb + "%";
        app.set_spiral_blend(sb / 100);

        app.set_theme(themeEl.value);
        app.set_light_mode(!darkQuery.matches);

        bloomActive = bloomEnabled.checked;
        bloomCanvas.style.display = bloomActive ? "block" : "none";
        if (bloomActive) {
            sizeBloomCanvas();
            bloomCanvas.style.opacity = parseInt(bloomIntensity.value) / 100;
        }
        bloomVal.textContent = parseInt(bloomIntensity.value) + "%";
    }

    const darkQuery = window.matchMedia("(prefers-color-scheme: dark)");
    function applySystemTheme() {
        app.set_light_mode(!darkQuery.matches);
        ensureLoop();
    }
    darkQuery.addEventListener("change", applySystemTheme);

    const controlsEl = document.getElementById("controls");
    const controlsToggle = document.getElementById("controls-toggle");
    controlsToggle.addEventListener("click", () => {
        controlsEl.classList.toggle("collapsed");
        controlsToggle.innerHTML = controlsEl.classList.contains("collapsed") ? "&#9650;" : "&#9660;";
        setTimeout(() => {
            let { w, h } = sizeCanvas();
            app.resize(w, h);
            if (bloomActive) sizeBloomCanvas();
            ensureLoop();
        }, 50);
    });

    syncAll();
    ensureLoop();
}

function buildRatioSelect(intervals) {
    const select = document.getElementById("ratio");
    const groups = {};

    for (const iv of intervals) {
        if (!groups[iv.group]) {
            groups[iv.group] = [];
        }
        groups[iv.group].push(iv);
    }

    for (const [groupName, items] of Object.entries(groups)) {
        const optgroup = document.createElement("optgroup");
        optgroup.label = groupName;
        for (const iv of items) {
            const opt = document.createElement("option");
            opt.value = `${iv.numerator}/${iv.denominator}`;
            opt.textContent = `${iv.numerator}/${iv.denominator} - ${iv.name}`;
            opt.title = `${iv.name} (~${Math.round(iv.cents)} cents)`;
            if (iv.numerator === 3 && iv.denominator === 2) {
                opt.selected = true;
            }
            optgroup.appendChild(opt);
        }
        select.appendChild(optgroup);
    }

    const customOpt = document.createElement("option");
    customOpt.value = "custom";
    customOpt.textContent = "Custom…";
    select.appendChild(customOpt);
}

main();
