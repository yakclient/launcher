import type {AppProps} from 'next/app'
import 'bootstrap/dist/css/bootstrap.min.css'
import './globals.scss'
import {Alert, ThemeProvider} from "react-bootstrap";
import React, {ReactNode, useEffect, useState} from "react";
import {Variant} from "react-bootstrap/types";

const ALERT_TIMEOUT = 10

// MSAL configuration

export interface LauncherAlert {
    variant: Variant,
    content: React.ReactNode,
    id: number
}

type AddAlert = (variant: Variant, content: ReactNode) => void

export const Alerts = React.createContext<AddAlert>(() => {});

export default function MyApp({Component, pageProps}: AppProps) {
    const [alertCount, setAlertCount] = useState(0);
    const [alerts, setAlerts] = useState<LauncherAlert[]>([]);


    const addAlert: AddAlert = (variant: Variant, content: ReactNode) => {
        let id = alertCount + 1;
        const alert = {
            variant: variant,
            content: content,
            id: id
        }

        setAlertCount(id)

        alerts.push(alert)

        const timer = setTimeout(() => {
            setAlerts((newAlerts) => newAlerts.filter((t) => t.id != id))
        }, ALERT_TIMEOUT * 1000);
    }

    return <div data-bs-theme="dark">
        <ThemeProvider>
            <Alerts.Provider value={addAlert}>
                <Component {...pageProps} />
                <div
                    style={{
                        position: "absolute",
                        bottom: 0,
                        right: 0,
                        zIndex: 20,
                        margin: "10px",
                        transition: "ease-in"
                    }}
                >
                    {alerts.map((value, index) =>
                        <Alert key={index} variant={value.variant} dismissible>
                            {value.content}
                        </Alert>
                    )}
                </div>
            </Alerts.Provider>
        </ThemeProvider>
    </div>
}

