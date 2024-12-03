import React, {useState} from "react";
import Nav from "@/components/nav";
import {Card} from "react-bootstrap";
import Extensions from "@/components/extension/extensions_search";
import Mods from "@/components/mod/mods_search";

const Market: React.FC = () => {
    let [page, setPage] = useState(0);

    let pages = [
        {
            name: "Extensions",
            component: <>
                <Extensions/>
            </>
        },
        {
            name: "Mods",
            component: <>
                <Mods/>
            </>
        }
    ]

    return <>
        <Nav
            elements={pages.map(({name}) => {
                return {
                    name: name
                }
            })}
            fontSize ="15px"
            color = "#bc4731"
            onChange={(index) => {
                setPage(index)
            }}
        ></Nav>
        {pages[page].component}
    </>
}

export default Market;