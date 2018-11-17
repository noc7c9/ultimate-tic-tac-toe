import React from 'react';
import styled, { createGlobalStyle } from 'styled-components';

const range = length => [...Array(length).keys()];
const grid3x3 = range(3).flatMap(y => range(3).map(x => ({ x, y })));

const colors = {
    highlight: '#ddc4de',
    x: {
        normal: '#1676b4',
        dark1: '#06568a',
        dark2: '#033d63',
        light1: '#438dbd',
        light2: '#82b5d6',
    },
    o: {
        normal: '#ff9611',
        dark1: '#d87a00',
        dark2: '#9a5700',
        light1: '#ffb350',
        light2: '#ffd195',
    },
}

export const GlobalStyles = createGlobalStyle`
    html, body {
        padding: 0;
        margin: 0;
    }
    #root {
        height: 100vh;
    }
`;

const GridContainer = styled.div`
    display: grid;
    grid-template: repeat(3, 33.33333%) / repeat(3, 33.33333%);
    grid-auto-flow: column;
    height: 100%;
`;

const GridCell = styled.div`
    position: relative;
    border: solid black 0;
    border-right-width: ${({ y, borderWidth = '1px' }) => y < 2 ? borderWidth : 0}
    border-bottom-width: ${({ x, borderWidth = '1px' }) => x < 2 ? borderWidth : 0}
`;

function Grid(props) {
    const {
        children = [],
        borderWidth,
    } = props;
    return (
        <GridContainer>
            {grid3x3.map(({ x, y }, i) => {
                const child = children[i];
                return (
                    <GridCell key={i} x={x} y={y}
                        borderWidth={borderWidth}
                        onClick={child.onClick}
                    >
                        {child}
                    </GridCell>
                );
            })}
        </GridContainer>
    );
}

export const OuterGrid = props => <Grid borderWidth="3px" {...props} />;

const InnerGridContainer = styled.div`
    box-sizing: border-box;
    width: 100%;
    height: 100%;
    padding: 15px;
    background: ${props => props.background}
`;
export const InnerGrid = props => (
    <InnerGridContainer background={props.highlight && colors.highlight}>
        <Grid {...props} />
    </InnerGridContainer>
);

const MarkSVG = styled.svg`
    fill: ${props => props.fill};
    background: ${({ background, overlay }) => `${background}${overlay ? '80' : ''}`};

    ${({ overlay }) => {
        const padding = overlay ? '15px' : '10px';
        return `
            ${overlay && `
                position: absolute;
                top: 0;
                left: 0;
            `};

            width: calc(100% - 2 * ${padding});
            height: calc(100% - 2 * ${padding});
            padding: ${padding};
        `
    }}
`;

export function X(props) {
    return (
        <MarkSVG {...props} viewBox="0 0 100 100"
            fill={colors.x.normal} background={colors.x.light2}
        >
            <path d="M 100,0 59.463724,49.429148 99.921137,100 H 81.861201 L 49.842271,58.831431 17.0347,100 H 0 L 40.9306,50.033579 0.94637224,0 H 18.927445 L 50.552054,40.631296 82.886436,0 Z" />
        </MarkSVG>
    );
}

export function O(props) {
    return (
        <MarkSVG {...props} viewBox="0 0 100 100"
            fill={colors.o.normal} background={colors.o.light2}
        >
            <path d="M 50,0 C 22.385763,0 0,22.385763 0,50 0,77.614237 22.385763,100 50,100 77.614237,100 100,77.614237 100,50 100,22.385763 77.614237,0 50,0 Z m 0,13 C 70.98682,13 87,29.01318 87,50 87,70.98682 70.98682,87 50,87 29.01318,87 13,70.98682 13,50 13,29.01318 29.01318,13 50,13 Z" />
        </MarkSVG>
    );
}

export const Empty = styled.div`
    width: 100%;
    height: 100%;
    ${props => props.clickable && `
        &: hover {
            background: ${colors[props.clickable].light2};
        }
    `}
`;

const OuterContainer = styled.div`
    display: flex;
    justify-content: center;
    align-items: center;
    width: 100vw;
    height: 100vh;
`;
const InnerContainer = styled.div`
    width: 100vmin;
    height: 100vmin;
`;
export function Page({ children }) {
    return (
        <OuterContainer>
            <InnerContainer>
                { children }
            </InnerContainer>
        </OuterContainer>
    );
}
