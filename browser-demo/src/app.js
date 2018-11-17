import React from 'react';
import { useState } from 'react';
import * as ultimateTicTacToe from '../../wasm_bindings/pkg';
import {
    GlobalStyles,
    Title, Page,
    OuterGrid, InnerGrid,
    X, O, Empty,
} from './components';

const coordToKey = ({ ox, oy, ix, iy }) => [ox, oy, ix, iy].join(',');

const gridMap = (grid, fn) => grid.flatMap(
    (row, x) => row.map((cell, y) => fn(cell, x, y)));

const useGame = () => {
    const [game, setGame] = useState(ultimateTicTacToe.initialize);
    const moves = ultimateTicTacToe.getMoves(game).reduce((map, move) => {
        map[coordToKey(move)] = move;
        return map;
    }, {});

    return [game, moves, (coord) => {
        const move = moves[coordToKey(coord)];
        if (move) {
            setGame(ultimateTicTacToe.playMove(game, move));
        }
    }];
}

export default () => {
    const [game, moves, playMove] = useGame();

    const cells = gridMap(game.grid, ({grid, completed}, ox, oy) => {
        const key = coordToKey({ ox, oy });

        const isActive = (
            game.activeOuterSquare?.x === ox && game.activeOuterSquare?.y === oy
        );
        const innerGrid = (
            <InnerGrid highlight={isActive} key={key}>
                {gridMap(grid, (cell, ix, iy) => {
                    const key = coordToKey({ ox, oy, ix, iy });
                    switch (cell) {
                        case "o":
                            return <O key={key} />
                        case "x":
                            return <X key={key} />
                        default:
                            return <Empty
                                onClick={() => playMove({ ox, oy, ix, iy })}
                                clickable={moves[key] ? game.turn : false}
                                key={key}
                            />;
                    }
                })}
            </InnerGrid>
        );

        switch (completed) {
            case "x":
                return (
                    <React.Fragment key={key} >
                        {innerGrid}
                        <X overlay />
                    </React.Fragment>
                );
            case "o":
                return (
                    <React.Fragment key={key} >
                        {innerGrid}
                        <O overlay />
                    </React.Fragment>
                );
            default:
                return innerGrid;
        }
    });

    return (
        <>
            <GlobalStyles />
            <Page>
                <OuterGrid>
                    {cells}
                </OuterGrid>
            </Page>
        </>
    );
}
