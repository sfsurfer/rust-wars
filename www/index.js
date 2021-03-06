import { Game, TurnPhase } from "rust-wars";

const MAP_SCALE = 20;

const BORDER_COLOR = "#DDDDDD";
const GRID_COLOR = "#CCCCCC";

const game = Game.new();
const map = game.get_map();
const width = map.width();
const height = map.height();
const territoryCount = map.territory_count();
const nodes = map.centers();


const getEventCoordinates = (canvas, event) => {
    const rect = canvas.getBoundingClientRect();
    const x = event.clientX - rect.left;
    const y = event.clientY - rect.top;
    return {x: x, y: y};
};

const territoryFromCoordinates = (coordinates) => {
    const colorArray = bgContext.getImageData(coordinates.x, coordinates.y, 1, 1).data
    const colorU32 = (colorArray[0] << 16) + (colorArray[1] << 8) + colorArray[2];
    return map.territory_with_color(colorU32);
};

const u32ToColor = (u32) => {
    return "#" + u32.toString(16).padStart(6,'0');
};

const bgCanvas = document.getElementById("rust-wars-bg");
bgCanvas.width = (width - 1) * MAP_SCALE;
bgCanvas.height = (height - 1) * MAP_SCALE;


const mapCanvas = document.getElementById("rust-wars-map");
mapCanvas.width = (width - 1) * MAP_SCALE;
mapCanvas.height = (height - 1) * MAP_SCALE;

const troopCanvas = document.getElementById("rust-wars-troop-boxes");
troopCanvas.width = (width - 1) * MAP_SCALE;
troopCanvas.height = (height - 1) * MAP_SCALE;
troopCanvas.addEventListener('mousedown', function (e) {
    let coordinates = getEventCoordinates(troopCanvas, e);
    let territory = territoryFromCoordinates(coordinates);
    let result = game.map_click_action(territory);
    if (result) renderLoop();
});

const bgContext = bgCanvas.getContext('2d');
bgContext.fillStyle = u32ToColor(map.background_color());
bgContext.fillRect(0,0,mapCanvas.width, mapCanvas.height);

const mapContext = mapCanvas.getContext('2d');

const troopContext = troopCanvas.getContext('2d');

troopContext.globalCompositeOperation = 'destination-over';

const initializeSelector = (selector) => {
    if (selector.nodeName && selector.nodeName.toLowerCase() === "select") {
        let len = selector.options.length;
        for (let i = len - 1; i >= 0; i--) {
            selector.remove(i);
        }
    } else {
        console.error("Attempted to call initializeSelector on type " + selector.nodeName);
    }
};

// --- TROOP PLACEMENT ---
const hidePlacementElements = (hide) => {
    if (typeof hide === "boolean") {
        clearPlacementButton.hidden = hide;
        applyPlacementButton.hidden = hide;
        placeTroopSelector.hidden = hide;
        troopCounterDisplay.hidden = hide;
    }
};

let clearPlacementButton = document.getElementById("clear-placement")
clearPlacementButton.addEventListener('click', event => {
    game.clear_placement_cache();
    renderLoop();
});

let applyPlacementButton = document.getElementById("apply-placement")
applyPlacementButton.addEventListener('click', e => {
    game.commit_placement_cache();
    if (game.troops_available_for_placement() === 0) {
        game.attack_phase();
        hidePlacementElements(true);
        placeButton.disabled = true;
    }
    renderLoop();
});

let placeButton = document.getElementById('placement-button')
placeButton.addEventListener('click', e => {
    game.place_phase();
    initializePlacementSelector();
    hidePlacementElements(false);
});

// Troops to place selector
const placeTroopSelector = document.getElementById("troops-to-place-selector");
const initializePlacementSelector = () => {
    initializeSelector(placeTroopSelector);
    let troops = game.troops_available_for_placement(); // - game.troops_staged_for_placement();
    for (const i of Array(troops).keys()) {
        placeTroopSelector.options[placeTroopSelector.options.length] = new Option((i+1).toString(), (i+1).toString());
    }
    placeTroopSelector.selectedIndex = 0;
};

const updatePlacementSelector = () => {
    placeTroopSelector.selectedIndex = game.get_troops_to_place() - 1;
};

placeTroopSelector.onchange = () => {
    let availableTroops = game.troops_available_for_placement(); //  - game.troops_staged_for_placement();
    if (placeTroopSelector.options.length >= availableTroops) {
        game.set_troops_to_place(1);
    } else {
        game.set_troops_to_place(placeTroopSelector.value);
    }
};

let attackButton = document.getElementById("attack-button")
attackButton.addEventListener('click', e => {
    game.attack_phase();
    hidePlacementElements(true);
})

let fortifyButton = document.getElementById("fortify-button")
fortifyButton.addEventListener('click', e => {
    game.fortify_phase();
    hidePlacementElements(true);
})

let endTurnButton = document.getElementById('end-turn-button')
endTurnButton.addEventListener('click', e => {
    game.init_turn();
    renderLoop();
})
const troopCounterDisplay = document.getElementById('troop-placement-counter')
const updateTroopPlacementCounter = () => {
    if (game.is_place_phase()) {
        troopCounterDisplay.hidden = false;
        let denominator = game.new_troops(); // TODO
        let numerator = game.troops_staged_for_placement();
        troopCounterDisplay.innerText = numerator.toString() + "/" + denominator.toString();
    } else {
        troopCounterDisplay.hidden = true;
    }
}

const getIndex = (row, column) => {
    return row * width + column;
};
const getX = (idx) => {
    return idx % width;
}
const getY = (idx) => {
    return  Math.floor(idx / width);
}

const gameStatus = () => {
    // Simply checks if game is over
    if (game.is_over()) {
        let winner = game.active_players()[0];
        alert(`Game Over. Player ${winner} won!`);
    }
}

const drawMapBackground = () => {
    const w = width - 1;
    const h = height - 1;

    bgContext.beginPath();
    bgContext.strokeStyle = BORDER_COLOR;

    // Draw border
    bgContext.moveTo(0,0);
    bgContext.lineTo(w * MAP_SCALE, 0);
    bgContext.lineTo(w * MAP_SCALE, h * MAP_SCALE);
    bgContext.lineTo(0, h * MAP_SCALE);
    bgContext.lineTo(0, 0);

    bgContext.stroke();

    for (let i = 0; i < territoryCount; i++) {
        let vertices = map.vertices_for(i);
        let rustColor = map.bg_color_for(i);
        // drawUpdate(vertices, rustColor);
        let color = u32ToColor(rustColor);

        bgContext.beginPath();
        bgContext.strokeStyle = GRID_COLOR;

        let start = vertices[0];
        bgContext.moveTo(getX(start) * MAP_SCALE, getY(start) * MAP_SCALE);

        for (let j = 1; j < vertices.length - 1; j++) {
            const p2 = vertices[j];
            bgContext.lineTo(getX(p2) * MAP_SCALE, getY(p2) * MAP_SCALE);
        }
        bgContext.closePath();
        bgContext.fillStyle = color;
        bgContext.fill();
        bgContext.stroke();
    }
};
const drawUpdate = (vertices, rustColor) => {
    let color = u32ToColor(rustColor);

    mapContext.beginPath();
    mapContext.strokeStyle = GRID_COLOR;

    let start = vertices[0];
    mapContext.moveTo(getX(start) * MAP_SCALE, getY(start) * MAP_SCALE);

    for (let j = 1; j < vertices.length - 1; j++) {
        const p2 = vertices[j];
        mapContext.lineTo(getX(p2) * MAP_SCALE, getY(p2) * MAP_SCALE);
    }
    mapContext.closePath();
    mapContext.fillStyle = color;
    mapContext.fill();
    mapContext.stroke();

}
const drawMap = () => {
    for (let i = 0; i < territoryCount; i++) {

        let vertices = map.vertices_for(i);
        let rustColor = game.get_map().color_for(i);
        // drawUpdate(vertices, rustColor);
        let color = u32ToColor(rustColor);

        mapContext.beginPath();
        mapContext.strokeStyle = GRID_COLOR;

        let start = vertices[0];
        mapContext.moveTo(getX(start) * MAP_SCALE, getY(start) * MAP_SCALE);

        for (let j = 1; j < vertices.length - 1; j++) {
            const p2 = vertices[j];
            mapContext.lineTo(getX(p2) * MAP_SCALE, getY(p2) * MAP_SCALE);
        }
        mapContext.closePath();
        mapContext.fillStyle = color;
        mapContext.fill();
        mapContext.stroke();
    }
};

const TROOP_RADIUS = MAP_SCALE * 0.65;
const drawTroopContainers = () => {
    for (let i = 0; i < territoryCount; i++) {
        let node = nodes[i];
        if (game.get_map().is_highlighted(i)) {
            let color = u32ToColor(game.get_map().color_for(i));
            mapContext.strokeStyle = color; //GRID_COLOR;
            mapContext.fillStyle = color;
            let x = getX(node) * MAP_SCALE;
            let y = getY(node) * MAP_SCALE;
            mapContext.beginPath();
            mapContext.arc(x, y, TROOP_RADIUS, 0, 2 * Math.PI);
            mapContext.shadowBlur = 15;
            mapContext.shadowColor = "#000000";
            mapContext.closePath();
            mapContext.fill();
            mapContext.stroke();
        }
    }
    mapContext.shadowBlur = 0;
};

const drawMovementArrow = () => {
    let map = game.get_map();
    if (map.movement_eminent()) {
        let arrow_start = map.get_movement_arrow_start();
        let arrow_end = map.get_movement_arrow_end();
        let x1 = getX(arrow_start) * MAP_SCALE;
        let y1 = getY(arrow_start) * MAP_SCALE;
        let x2 = getX(arrow_end) * MAP_SCALE ;
        let y2 = getY(arrow_end) * MAP_SCALE ;

        drawArrow(x1, y1, x2, y2);
    }
}

const drawArrow = (fromx, fromy, tox, toy) => {
    let RADIUS_MULTIPLIER = 1.25
    let headlen = 10;
    let xDiff = tox - fromx;
    let yDiff = toy - fromy;
    let angle = Math.atan2(yDiff, xDiff);

    let x1 = fromx + (TROOP_RADIUS * RADIUS_MULTIPLIER) * Math.cos(angle);
    let y1 = fromy + (TROOP_RADIUS * RADIUS_MULTIPLIER) * Math.sin(angle);
    let x2 = tox - (TROOP_RADIUS * RADIUS_MULTIPLIER) * Math.cos(angle);
    let y2 = toy - (TROOP_RADIUS * RADIUS_MULTIPLIER) * Math.sin(angle);

    mapContext.strokeStyle = 'white';
    mapContext.lineWidth = 2;
    mapContext.beginPath();
    mapContext.moveTo(x1, y1);
    mapContext.lineTo(x2, y2);
    mapContext.lineTo(x2 - headlen * Math.cos(angle - Math.PI / 6), y2 - headlen * Math.sin(angle - Math.PI / 6));
    mapContext.lineTo(x2 - headlen * Math.cos(angle + Math.PI / 6), y2 - headlen * Math.sin(angle + Math.PI / 6));
    mapContext.lineTo(x2, y2);
    mapContext.closePath();
    mapContext.fillStyle = 'white';
    mapContext.fill();
    mapContext.stroke();
}

// -- ATTACKING -- //
const attackModal = document.getElementById("attack-modal");
const attackTroopSelector = document.getElementById("troop-attack-selector");
attackTroopSelector.onchange = () => {
    game.attack_with(attackTroopSelector.selectedIndex + 1);
    attackModal.style.zIndex = -1;
    renderLoop();
}
const attackAndTailButton = document.getElementById('attack-modal-button-tail');
attackAndTailButton.addEventListener('click', e => {
    game.attack_tail();
    renderLoop();
});
const attackAllButton = document.getElementById('attack-modal-button-all');
attackAllButton.addEventListener('click', e => {
    game.attack_all();
    renderLoop();
});

const attackCancelButton = document.getElementById('attack-modal-button-cancel')
attackCancelButton.addEventListener('click', e => {
    attackModal.style.zIndex = -1;
    game.unselect_all();
    renderLoop();
})
const showAttackPrompt = () => {
    // Initialize options
    let len = attackTroopSelector.options.length;
    for (let i = len - 1; i >= 0; i-- ) {
        attackTroopSelector.remove(i);
    }
    let troops = game.troops_available_for_movement();
    // attackTroopSelector.options.length = troops;
    for (const i of Array(troops).keys()) {
        if (i === 0) continue;
        attackTroopSelector.options[attackTroopSelector.options.length] = new Option(i.toString(),i.toString());
    }
    attackModal.style.zIndex = 9999;
}
const hideAttackPrompt = () => {
    attackModal.style.zIndex = -1;
}

// --- FORTIFY ---
const fortifyModal = document.getElementById("fortify-modal");
const fortifyTroopSelector = document.getElementById("troop-fortify-selector");
fortifyTroopSelector.onchange = () => {
    game.fortify_troops(fortifyTroopSelector.selectedIndex + 1);
    fortifyModal.style.zIndex = -1;
    renderLoop();
}
const fortifyAllButton = document.getElementById('fortify-modal-button-all');
fortifyAllButton.addEventListener('click', e => {
    game.fortify_all();
    fortifyModal.style.zIndex = -1;
    renderLoop();
})
const showFortifyPrompt = (troops = game.troops_available_for_movement()) => {
    // Initialize options
    initializeSelector(fortifyTroopSelector);
    for (const i of Array(troops).keys()) {
        if (i === 0) continue;
        fortifyTroopSelector.options[fortifyTroopSelector.options.length] = new Option(i.toString(),i.toString());
    }
    fortifyModal.style.zIndex = 9999;
}
const hideFortifyPrompt = () => {
    fortifyModal.style.zIndex = -1;
}

const updateTroops = () => {
    let troops = game.get_map().troops();
    mapContext.fillStyle = 'white';
    mapContext.font = 'bold 12px Arial';
    mapContext.textAlign = 'center';
    mapContext.strokeStyle = 'black';
    mapContext.lineWidth = 1;

    for (let i = 0; i < territoryCount; i++) {
        let node = nodes[i];
        let n = troops[i].toString();
        let x = getX(node) * MAP_SCALE;
        let y = getY(node) * MAP_SCALE;

        mapContext.beginPath();
        mapContext.textBaseline = 'middle';
        mapContext.textAlign = 'center';
        mapContext.fillText(n, x, y);
    }
};
const updateControls = () => {
    initializePlacementSelector();
    updateTroopPlacementCounter();
    if (game.is_place_phase()) {
        hidePlacementElements(false);
    }
    if (game.is_attack_phase() && game.target_selected()) {
        showAttackPrompt();
    } else {
        hideAttackPrompt();
    }
    if (game.is_fortify_phase() && game.target_selected()) {
        showFortifyPrompt();
    } else {
        hideFortifyPrompt();
    }
}
const renderLoop = () => {
    // mapContext.globalCompositeOperation = 'destination-over';
    drawMap();
    drawTroopContainers();
    drawMovementArrow();
    updateTroops();
    updateControls();
    gameStatus(); // todo: restart/disable game after someone wins
};

drawMapBackground();
requestAnimationFrame(renderLoop);
