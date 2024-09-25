import React, {useState} from "react";
import Nav from "@/components/nav";
import {Card} from "react-bootstrap";
import Extensions from "@/components/extensions_search";

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
                <Card style={{
                    margin: "10px 0",
                    maxHeight: "200px",
                }}>
                    <Card.Body>
                        <Card.Title as="h4" className="h5 h4-sm">
                            Not done yet...
                        </Card.Title>
                        <Card.Text>What are you waiting for? I told you this wasnt done.</Card.Text>

                    </Card.Body>
                </Card>
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