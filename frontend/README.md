# Ar3mis Cloud Frontend

Next.js 14 frontend for the Ar3mis Cloud self-hosted storage platform.

## Tech Stack
- **Framework**: Next.js 14 (App Router)
- **Language**: TypeScript
- **Styling**: Tailwind CSS
- **State Management**: Zustand with persistence
- **HTTP Client**: Axios with interceptors
- **Notifications**: react-hot-toast
- **Icons**: Lucide React

## Features
- User authentication (login/register with JWT)
- File browser with folder navigation
- Drag-and-drop file upload with chunked upload support
- Real-time upload progress
- File operations (create folder, delete, download, share)
- Responsive design with mobile sidebar
- Automatic token refresh

## Project Structure
```
src/
├── app/                    # Next.js App Router pages
│   ├── page.tsx           # Redirect to login/dashboard
│   ├── login/page.tsx     # Login page
│   └── dashboard/page.tsx # Main dashboard with file browser
├── components/
│   ├── FileList.tsx       # File browser component
│   └── LoginForm.tsx      # Authentication form
├── lib/
│   ├── api.ts             # Axios API client with auth interceptors
│   └── store.ts           # Zustand stores (auth, files, uploads)
└── globals.css            # Global styles + Tailwind
```

## Development

```bash
# Install dependencies
npm install

# Run development server
npm run dev

# Build for production
npm run build

# Start production server
npm start
```

## Environment Variables
Create `.env.local` for development:
```env
NEXT_PUBLIC_API_URL=http://localhost:8000
```

For production (Vercel):
```env
NEXT_PUBLIC_API_URL=https://your-domain.com
```

## API Integration
The frontend communicates with the backend via:
- REST API at `/api/v1/*` endpoints
- Chunked uploads for large files
- JWT authentication with automatic refresh

## Deployment to Vercel
1. Push to GitHub
2. Import project in Vercel
3. Set `NEXT_PUBLIC_API_URL` environment variable
4. Deploy

The `next.config.js` includes rewrites to proxy API calls in development.